use crate::EventLog;
use evtx::EvtxParser;
use serde_json::Value;
use std::path::PathBuf;

pub fn get_event_log(input_path: String) -> Option<EventLog> {
    const EVENT_PATH: &str = "Event.System.Channel";
    const HOSTNAME_PATH: &str = "Event.System.Computer";

    let evtx_file = PathBuf::from(input_path);
    let mut parser = EvtxParser::from_path(&evtx_file).expect("Failed to open EVTX file");
    for event in parser.records_json() {
        match event {
            Ok(evt) => {
                let json_object: Value = serde_json::from_str(evt.data.as_str())
                    .expect("Failed to parse JSON from event data");

                let log_channel = crate::extract_json_field(&json_object, EVENT_PATH)
                    .unwrap_or_else(|| "Unknown".to_string());
                let host_name = crate::extract_json_field(&json_object, HOSTNAME_PATH)
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