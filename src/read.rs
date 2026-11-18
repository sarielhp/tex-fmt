//! Utilities for reading files

use crate::logging::{record_file_log, Log};
use crate::regexes::EXTENSIONS;
use log::Level::{Error, Trace};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

/// Read a file with an accepted extension
pub fn read(file: &PathBuf, logs: &mut Vec<Log>) -> Option<String> {
    // Check if file has an accepted extension
    let has_ext = file
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONS.contains(&e));

    let result = fs::read_to_string(file);
    if let Ok(text) = result {
        return Some(text);
    }
    if let (true, Err(e)) = (has_ext, result) {
        record_file_log(
            logs,
            Error,
            file,
            &format!("Could not open file: {e}"),
        );
    } else {
        record_file_log(logs, Error, file, "File type invalid.");
    }
    None
}

/// Attempt to read from stdin, return filename `<stdin>` and text
pub fn read_stdin(logs: &mut Vec<Log>) -> Option<String> {
    let mut text = String::new();
    let stdin_path = PathBuf::from("<stdin>");
    match std::io::stdin().read_to_string(&mut text) {
        Ok(bytes) => {
            record_file_log(
                logs,
                Trace,
                &stdin_path,
                &format!("Read {bytes} bytes."),
            );
            Some(text)
        }
        Err(e) => {
            record_file_log(
                logs,
                Error,
                &stdin_path,
                &format!("Could not read from stdin: {e}"),
            );
            None
        }
    }
}
