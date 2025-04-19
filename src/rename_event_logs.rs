use crate::{get_event_log, EventLog};
use std::fs;
use std::path::PathBuf;

pub fn rename_event_logs(input_path: String) {
    let old_path = &input_path;
    let event_log: EventLog = get_event_log::get_event_log(old_path.to_string()).clone().unwrap();
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