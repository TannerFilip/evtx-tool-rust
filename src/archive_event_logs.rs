use chrono::{DateTime, Utc};
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tar::Archive;
use tar::Builder;
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

use crate::find_event_logs::find_event_logs;

pub fn archive_event_logs(input_path: String, output_path: PathBuf, dry_run: bool) {
    // Check if output_path exists
    if !PathBuf::from(&output_path).is_dir() {
        if dry_run {
            println!("[DRY RUN] Would create directory: {:?}", output_path);
        } else {
            fs::create_dir_all(&output_path).unwrap();
        }
    }

    println!("{}Outputting to {:?}", if dry_run { "[DRY RUN] " } else { "" }, output_path);

    let now: DateTime<Utc> = Utc::now();
    let timestamp = now.format("%Y-%m-%dT%H_%M_%SZ").to_string();
    let event_logs = find_event_logs(input_path);

    // Create the output filename with timestamp
    let output_filename = format!("event_logs_{}.tar.xz", timestamp);
    let output_file_path = output_path.join(output_filename);

    if dry_run {
        println!("[DRY RUN] Would create archive: {:?}", output_file_path);
        println!("[DRY RUN] Would archive the following files:");
        for log in &event_logs {
            println!("  - {}", log.display());
        }
        println!("[DRY RUN] Would verify archive contents");
        println!("[DRY RUN] Would delete original files after successful verification");
        return;
    }

    // Create the tar.xz archive
    match create_tar_xz(event_logs.clone(), &output_file_path) {
        Ok(_) => {
            println!("Successfully created archive: {:?}", output_file_path);
            
            // Verify the archive contents
            if verify_archive_contents(&output_file_path, &event_logs) {
                println!("Archive verification successful");
                // Delete original files
                for log in event_logs {
                    if let Err(e) = fs::remove_file(&log) {
                        eprintln!("Failed to delete {}: {}", log.display(), e);
                    } else {
                        println!("Deleted original file: {}", log.display());
                    }
                }
            } else {
                eprintln!("Archive verification failed - original files preserved");
            }
        }
        Err(e) => eprintln!("Failed to create archive: {}", e),
    }
}

fn verify_archive_contents(archive_path: &PathBuf, original_files: &[PathBuf]) -> bool {
    let file = match File::open(archive_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open archive for verification: {}", e);
            return false;
        }
    };

    let decoder = XzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    
    let mut archived_files: Vec<PathBuf> = Vec::new();
    
    for entry in archive.entries().unwrap() {
        if let Ok(entry) = entry {
            if let Ok(path) = entry.path() {
                archived_files.push(path.into_owned());
            }
        }
    }

    // Check if all original files are in the archive
    let mut missing_files = Vec::new();
    for original in original_files {
        let file_name = original.file_name().unwrap();
        if !archived_files.iter().any(|p| p.file_name().unwrap() == file_name) {
            missing_files.push(original.display().to_string());
        }
    }

    if !missing_files.is_empty() {
        eprintln!("The following files are missing from the archive:");
        for file in missing_files {
            eprintln!("  - {}", file);
        }
        return false;
    }

    true
}

pub fn create_tar_xz<P: AsRef<std::path::Path>>(paths: Vec<PathBuf>, output_path: P) -> io::Result<()> {
    let output_file = File::create(output_path)?;
    let xz_encoder = XzEncoder::new(output_file, 9);
    let mut tar_builder = Builder::new(xz_encoder);

    for path in &paths {
        // Get the file name to use as the path in the archive
        let file_name = path.file_name().unwrap();
        let mut file = File::open(path)?;
        tar_builder.append_file(Path::new(&file_name), &mut file)?;
    }

    let xz_encoder = tar_builder.into_inner()?;
    let _output_file = xz_encoder.finish()?;

    Ok(())
}