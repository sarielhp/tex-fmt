//! Utilities for performing text substitutions

use crate::args::Args;
use crate::comments::find_comment_index;
use crate::format::{Pattern, State};
use crate::logging::{record_line_log, Log};
use crate::regexes;
use crate::LINE_END;
use log::Level;
use log::LevelFilter;
use std::path::Path;

/// Remove multiple line breaks
#[must_use]
pub fn remove_extra_newlines(text: &str) -> String {
    let double_line_end = format!("{LINE_END}{LINE_END}");
    regexes::RE_NEWLINES
        .replace_all(text, double_line_end)
        .to_string()
}

/// Replace tabs with spaces
#[must_use]
pub fn remove_tabs(text: &str, args: &Args) -> String {
    let replace = (0..args.tabsize).map(|_| " ").collect::<String>();
    text.replace('\t', &replace)
}

/// Remove trailing spaces from line endings
#[must_use]
pub fn remove_trailing_spaces(text: &str) -> String {
    regexes::RE_TRAIL.replace_all(text, LINE_END).to_string()
}

/// Remove trailing blank lines from file
#[must_use]
pub fn remove_trailing_blank_lines(text: &str) -> String {
    let mut new_text = text.trim_end().to_string();
    new_text.push_str(LINE_END);
    new_text
}

/// Check if line contains content which be split onto a new line
///
/// # Panics
///
/// This function should not panic as we already check for a regex match
/// before finding the match index and unwrapping.
#[must_use]
pub fn needs_split(line: &str, pattern: &Pattern) -> bool {
    // Don't split anything if the line contains a \verb|...|
    if pattern.contains_verb {
        return false;
    }

    // Check if we should format this line and if we've matched an environment.
    let contains_splittable_env = pattern.contains_splitting
        && regexes::RE_SPLITTING_SHARED_LINE.is_match(line);

    // If we're not ignoring and we've matched an environment ...
    if contains_splittable_env {
        // ... return `true` if the comment index is `None`
        // (which implies the split point must be in text), otherwise
        // compare the index of the comment with the split point.
        find_comment_index(line, pattern).is_none_or(|comment_index| {
            if regexes::RE_SPLITTING_SHARED_LINE_CAPTURE
                .captures(line)
                .unwrap() // Matched split point so no panic.
                .get(2)
                .unwrap() // Regex has 4 groups so index 2 is in bounds.
                .start()
                > comment_index
            {
                // If split point is past the comment index, don't split.
                false
            } else {
                // Otherwise, split point is before comment and we do split.
                true
            }
        })
    } else {
        // If ignoring or didn't match an environment, don't need a new line.
        false
    }
}

/// Where to split a line which starts with `\begin{env}`, so that the text
/// after the environment's arguments goes on a new line
/// (`split-after-begin`)
///
/// The arguments are the `[...]` and `{...}` groups directly after
/// `\begin{env}`, as in `\begin{theorem}[Name]{Text}` or
/// `\begin{tabular}{p{3cm}l}`; a `[...]` group may follow spaces. There is
/// no split when nothing but a comment follows, when an argument is not
/// closed on the line, or for verbatim environments. A `\verb` after the
/// split point moves to the new line intact.
#[must_use]
pub fn split_after_begin_point(
    line: &str,
    pattern: &Pattern,
    args: &Args,
) -> Option<usize> {
    if !args.split_after_begin || !pattern.contains_env_begin {
        return None;
    }
    let start = line.len() - line.trim_start().len();
    let after = line[start..].strip_prefix(regexes::ENV_BEGIN)?;
    let env = &after[..after.find('}')?];
    if args.verbatims.iter().any(|v| v == env) {
        return None;
    }
    let bytes = line.as_bytes();
    let mut i = start + regexes::ENV_BEGIN.len() + env.len() + 1;
    loop {
        let j = i + bytes[i..]
            .iter()
            .take_while(|b| b.is_ascii_whitespace())
            .count();
        match bytes.get(j) {
            Some(b'[') => i = group_end(bytes, j, b']')?,
            Some(b'{') if j == i => i = group_end(bytes, j, b'}')?,
            _ => break,
        }
    }
    let text = line[i..].trim_start();
    (!text.is_empty() && !text.starts_with('%')).then_some(i)
}

/// The index after the `close` ending a group opened at `start`, skipping
/// nested braces and escaped characters, or `None` if the group is not
/// closed on the line
fn group_end(bytes: &[u8], start: usize, close: u8) -> Option<usize> {
    let mut depth = 0;
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b if b == close && depth == 0 => return Some(i + 1),
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    None
}

/// Ensure lines are split correctly.
///
/// Returns a tuple containing:
/// 1. a reference to the line that was given, shortened because of the split
/// 2. a reference to the part of the line that was split
///
/// # Panics
///
/// This function should not panic as all regexes are validated.
pub fn split_line<'a>(
    line: &'a str,
    state: &State,
    file: &Path,
    args: &Args,
    logs: &mut Vec<Log>,
) -> (&'a str, &'a str) {
    let captures = regexes::RE_SPLITTING_SHARED_LINE_CAPTURE
        .captures(line)
        .unwrap();

    let (line, [prev, rest, _]) = captures.extract();

    if args.verbosity == LevelFilter::Trace {
        record_line_log(
            logs,
            Level::Trace,
            file,
            state.linum_new,
            state.linum_old,
            line,
            "Placing environment on new line.",
        );
    }
    (prev, rest)
}
