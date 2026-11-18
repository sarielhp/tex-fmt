//! Utilities for finding, extracting and removing LaTeX comments

use crate::format::Pattern;

/// Commands whose first argument is a URL, in which `%` is not a comment
const URL_COMMANDS: [&str; 4] =
    ["\\url{", "\\href{", "\\nolinkurl{", "\\path{"];

/// Find the location where a comment begins in a line
///
/// A `%` starts a comment unless it is escaped as `\%`, or it is inside the
/// URL argument of `\url`, `\href`, `\nolinkurl` or `\path`, where raw
/// percent signs are allowed (#55). A `%` after `\\` does start a comment.
#[must_use]
pub fn find_comment_index(line: &str, pattern: &Pattern) -> Option<usize> {
    // Often there is no '%' so check this first
    if !pattern.contains_comment {
        return None;
    }
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => return Some(i),
            b'\\' => {
                let rest = &bytes[i..];
                i = match URL_COMMANDS
                    .iter()
                    .find(|cmd| rest.starts_with(cmd.as_bytes()))
                {
                    Some(cmd) => skip_group(bytes, i + cmd.len()),
                    // Skip the escaped character, so \% and \\ are not
                    // read again; '%' and '\' are ASCII, so this never
                    // lands inside a multi-byte character that matters
                    None => i + 2,
                };
            }
            _ => i += 1,
        }
    }
    None
}

/// The index just after the `}` closing a group whose `{` is before
/// `start`, or the end of the line if the group is not closed on it
fn skip_group(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

/// Remove a comment from the end of a line
#[must_use]
pub fn remove_comment(line: &str, comment: Option<usize>) -> &str {
    comment.map_or_else(|| line, |c| &line[0..c])
}
