//! Utilities for writing formatted files

use crate::args::Args;
use crate::logging::{record_file_log, Log};
use log::Level::{Error, Info};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Write a formatted file to disk, atomically where possible
///
/// The text is written to a temporary file next to the original, which
/// then replaces it, so a failed write (disk full, quota) leaves the
/// original file intact. Symbolic links are followed, so the link is kept
/// and the file it points to is replaced, with its permissions. A
/// read-only file is not replaced. If no temporary file can be created
/// next to the original (a read-only directory), the file is rewritten in
/// place instead.
fn write_file(file: &Path, text: &str) -> io::Result<()> {
    let filepath = file.canonicalize()?;
    let permissions = fs::metadata(&filepath)?.permissions();
    if permissions.readonly() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "file is read-only",
        ));
    }
    let name = filepath.file_name().unwrap_or_default().to_string_lossy();
    let temp = filepath
        .with_file_name(format!(".{name}.tex-fmt-{}.tmp", std::process::id()));
    let mut out = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
    {
        Ok(out) => out,
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            return fs::write(&filepath, text);
        }
        Err(e) => return Err(e),
    };
    let result = out
        .write_all(text.as_bytes())
        .and_then(|()| out.sync_all())
        .and_then(|()| fs::set_permissions(&temp, permissions))
        .and_then(|()| fs::rename(&temp, &filepath));
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

/// Print formatted text to standard output
fn print_text(text: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(text.as_bytes())?;
    out.flush()
}

/// Handle the newly formatted file
pub fn process_output(
    args: &Args,
    file: &Path,
    text: &str,
    new_text: &str,
    logs: &mut Vec<Log>,
) -> u8 {
    if args.print {
        if let Err(e) = print_text(new_text) {
            record_file_log(
                logs,
                Error,
                file,
                &format!("Could not print output: {e}"),
            );
            return 1;
        }
    } else if args.check && text != new_text {
        record_file_log(logs, Error, file, "Incorrect formatting.");
        return 1;
    } else if text != new_text {
        if let Err(e) = write_file(file, new_text) {
            record_file_log(
                logs,
                Error,
                file,
                &format!("Could not write file: {e}"),
            );
            return 1;
        }
        if args.fail_on_change {
            record_file_log(logs, Info, file, "Fixed incorrect formatting.");
            return 1;
        }
    }
    0
}
