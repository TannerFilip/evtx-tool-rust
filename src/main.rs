use clap::{Parser, Subcommand};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

mod find_event_logs;
mod get_event_log;
mod rename_event_logs;
mod archive_event_logs;

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
        /// Show what would be done without actually doing it
        #[arg(long)]
        dry_run: bool,
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
            dry_run,
        } => archive_event_logs(input_path, output_path, dry_run),
        Commands::Rename { input_path } => {
            rename_event_logs::rename_event_logs(input_path);
        }
        Commands::List { input_path } => {
            find_event_logs::find_event_logs(input_path);
        }
    }
}

fn extract_json_field(json: &Value, field_path: &str) -> Option<String> {
    field_path
        .split('.')
        .fold(Some(json), |acc, key| acc?.get(key))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

fn archive_event_logs(input_path: String, output_path: Option<String>, dry_run: bool) {
    let output_path = output_path.map(PathBuf::from).unwrap_or_else(|| {
        let user_home = get_user_home();
        user_home.join("evtx-archive")
    });
    archive_event_logs::archive_event_logs(input_path, output_path, dry_run);
}

fn get_user_home() -> PathBuf {
    let user_dirs = UserDirs::new();
    let user_home_dir = user_dirs.unwrap();
    user_home_dir.home_dir().to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_event_log;
    #[test]
    fn test_get_event_log() {
        let app_log = get_event_log::get_event_log("./samples/Application.evtx".to_string());
        assert_eq!(app_log.unwrap().event_log_type, "Application");
        let dirty_sec = get_event_log::get_event_log("./samples/2-system-Security-dirty.evtx".to_string());
        assert_eq!(dirty_sec.unwrap().event_log_type, "Security");
    }
    #[test]
    #[should_panic]
    fn test_get_event_log_not_evtx() {
        let _not_log_file = get_event_log::get_event_log("./samples/Catfractal.jpg".to_string());
    }
}
