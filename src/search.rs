//! Finding files in the file system

use crate::regexes::EXTENSIONS;
use ignore::Walk;
use std::path::PathBuf;

/// Find files recursively and append
pub fn find_files(dir: &PathBuf, files: &mut Vec<PathBuf>) {
    // Recursive walk of passed directory (ignore errors, symlinks and non-dirs)
    for entry in Walk::new(dir).filter_map(std::result::Result::ok) {
        // If entry is file and has accepted extension, push to files
        if entry.file_type().is_some_and(|t| t.is_file()) {
            let file = entry.path();
            let accepted = file
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| EXTENSIONS.contains(&e));
            if accepted {
                files.push(file.to_path_buf());
            }
        }
    }
}
