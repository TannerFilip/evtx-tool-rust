use clap::builder::Str;
use clap::{Parser, Subcommand};
use evtx::EvtxParser;
use infer::{MatcherType, Type};
use sanitize_filename;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::ops::Add;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EventLog {
    event_log_type: String,
    host_name: String,
    event_log_path: PathBuf,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Compresses evtx files as a __TODO__ file and saves them to an output directory
    Archive {
        /// Directory containing evtx files. Does not recurse.
        input_path: String,
        /// Output directory for single archive to be put into
        output_path: Option<String>,
    },
    /// Renames a given file to be HOSTNAME-LOGTYPE.evtx
    Rename {
        /// Path of file to rename
        input_path: String,
    },
    /// Lists evtx files in a given directory
    List {
        /// Directory to search for evtx files
        input_path: String,
    },
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Archive {
            input_path,
            output_path,
        } => archive_event_logs(input_path, output_path),
        Commands::Rename { input_path } => { rename_event_logs(input_path); } ,
        Commands::List { input_path } => { find_event_logs(input_path); }
    }
}

fn get_event_log(input_path: String) -> Option<EventLog> {
    const EVENT_PATH: &str = "Event.System.Channel";
    const HOSTNAME_PATH: &str = "Event.System.Computer";

    let evtx_file = PathBuf::from(input_path);
    let mut parser = EvtxParser::from_path(&evtx_file).expect("Failed to open EVTX file");
    for event in parser.records_json() {
        match event {
            Ok(evt) => {
                let json_object: Value = serde_json::from_str(evt.data.as_str())
                    .expect("Failed to parse JSON from event data");

                let log_channel = extract_json_field(&json_object, EVENT_PATH)
                    .unwrap_or_else(|| "Unknown".to_string());
                let host_name = extract_json_field(&json_object, HOSTNAME_PATH)
                    .unwrap_or_else(|| "Unknown".to_string());

                return Some(EventLog {
                    event_log_type: log_channel,
                    host_name,
                    event_log_path: evtx_file.clone(),
                });
            }
            Err(e) => {
                eprintln!("Failed to parse event record: {:?}", e);
            }
        }
    }

    None
}

fn extract_json_field(json: &Value, field_path: &str) -> Option<String> {
    field_path
        .split('.')
        .fold(Some(json), |acc, key| acc?.get(key))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

fn find_event_logs(input_path: String) -> Vec<PathBuf> {
    fn evtx_matcher(buf: &[u8]) -> bool {
        buf.len() >= 7
            && buf[0] == 0x45
            && buf[1] == 0x6C
            && buf[2] == 0x66
            && buf[3] == 0x46
            && buf[4] == 0x69
            && buf[5] == 0x6C
            && buf[6] == 0x65
    }
    let mut evtx_list = Vec::new();
    let mut info = infer::Infer::new();
    info.add("application/x-evtx", "evtx", evtx_matcher);

    let directory = fs::read_dir(input_path).unwrap();
    for file in directory {
        match file {
            Ok(f) => {
                let current_file_name = f.path();
                if current_file_name.is_file() {
                    let file_infer = info.get_from_path(&current_file_name);
                    match file_infer {
                        Ok(Some(file_type)) => match file_type.mime_type() {
                            "application/x-evtx" => evtx_list.push(current_file_name),
                            _ => {}
                        },
                        Err(e) => {
                            eprintln!("{:?}", e);
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
    return evtx_list;
}

fn archive_event_logs(input_path: String, output_path: Option<String>) {}

fn rename_event_logs(input_path: String) {
    let old_path = &input_path;
    let event_log: EventLog = get_event_log(old_path.to_string()).clone().unwrap();
    let cleaned_host_name = event_log.host_name.split('.').collect::<Vec<&str>>()[0];
    let new_file_name = format!("{}-{}", cleaned_host_name, event_log.event_log_type);
    let sanitized_file_name = sanitize_filename::sanitize(new_file_name + ".evtx");
    let binding = PathBuf::from(old_path);
    let working_dir = binding.parent().unwrap();
    println!("{:?}", working_dir);
    let new_path = PathBuf::from(working_dir).join(sanitized_file_name);
    match fs::rename(old_path, &new_path) {
        Ok(_) => println!("Renamed {:?} to {:?}", old_path, &new_path),
        Err(e) => println!("Error renaming file: {:?}", e),
    }
}
