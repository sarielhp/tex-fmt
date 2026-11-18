//! Formatting tables

use crate::format::{Pattern, State};
use crate::regexes::{TABLES_BEGIN, TABLES_END};
use crate::LINE_END;
use regex::Regex;

// Remove all double spaces from the table
fn remove_double_spaces(text: &str) -> String {
    let re = Regex::new(r"(\S) {2,}").unwrap();
    re.replace_all(text, "$1 ").to_string()
}

// Add line breaks after "\\"
fn add_line_breaks(text: &str) -> (String, bool) {
    let re_break = Regex::new(r"\\\\ .*\S").unwrap();
    let re_indent = Regex::new(r"^\s*\S").unwrap();
    let re_first_non_white = Regex::new(r"\S.*").unwrap();
    let re_to_break = Regex::new(r"^.*?\\\\").unwrap();
    let mut new_text = String::new();
    let mut finished: bool = true;
    for line in text.lines() {
        if re_break.is_match(line) {
            finished = false;
            let indent = re_indent.find(line).map_or("", |m| {
                let s = m.as_str();
                &s[..s.len() - 1]
            });
            let next_line_long = re_break.find(line).map_or("", |m| {
                let s = m.as_str();
                &s[2..]
            });
            let next_line = re_first_non_white
                .find(next_line_long)
                .map_or("", |m| m.as_str());
            let this_line = re_to_break.find(line).map_or("", |m| m.as_str());
            new_text.push_str(this_line);
            new_text.push_str(LINE_END);
            new_text.push_str(indent);
            new_text.push_str(next_line);
            new_text.push_str(LINE_END);
        } else {
            new_text.push_str(line);
            new_text.push_str(LINE_END);
        }
    }
    (new_text, finished)
}

// Get the starting positions of all delims, as character columns
fn get_positions(text: &str) -> Vec<Vec<usize>> {
    text.lines()
        .map(|l| {
            let mut prev = None;
            l.chars()
                .enumerate()
                .filter_map(|(i, c)| {
                    let is_match = c == '&' && prev != Some('\\');
                    prev = Some(c);
                    if is_match {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect()
}

// Find the desired new positions of the delims
fn get_new_positions(positions: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n_delims = positions.iter().map(std::vec::Vec::len).max().unwrap();
    let mut new_positions = positions.to_owned();
    for j in 0..n_delims {
        let positions: Vec<Option<usize>> =
            new_positions.iter().map(|l| l.get(j).copied()).collect();
        let new_position: usize = positions.iter().max().unwrap().unwrap();
        for line in &mut new_positions {
            let n_delims_line = line.len();
            if n_delims_line > j {
                let offset = new_position - line[j];
                for pos in line.iter_mut().skip(j) {
                    *pos += offset;
                }
            }
        }
    }
    new_positions
}

// Calculate the cumulative offsets needed
fn get_offsets(
    positions: &[Vec<usize>],
    new_positions: &[Vec<usize>],
) -> Vec<Vec<usize>> {
    let mut offsets = vec![];
    for l in 0..positions.len() {
        let mut prev_offset = 0;
        let mut offset = vec![];
        for j in 0..positions[l].len() {
            offset.push(new_positions[l][j] - positions[l][j] - prev_offset);
            prev_offset = new_positions[l][j] - positions[l][j];
        }
        offsets.push(offset);
    }
    offsets
}

// Use the offsets to format one line of the table
fn format_table_line(line: &str, offsets_delims_row: &[usize]) -> String {
    let mut new_line = String::new();
    let mut j = 0;
    let mut prev = None;
    for c in line.chars() {
        if c == '&' && prev != Some('\\') {
            let offset = offsets_delims_row.get(j).copied().unwrap_or(0);
            new_line.extend(std::iter::repeat_n(' ', offset));
            j += 1;
        }
        new_line.push(c);
        prev = Some(c);
    }
    new_line
}

// Format a single table
fn format_table(text: &str) -> String {
    let mut clean_text = remove_double_spaces(text);

    // add line breaks
    let mut finished: bool;
    (clean_text, finished) = add_line_breaks(&clean_text);
    let max_line_break_attempts = 10;
    for _attempt in 0..max_line_break_attempts {
        if finished {
            break;
        }
        (clean_text, finished) = add_line_breaks(&clean_text);
    }

    let positions = get_positions(&clean_text);
    let new_positions = get_new_positions(&positions);
    let offsets = get_offsets(&positions, &new_positions);
    let mut new_text = String::new();
    for (linum, line) in clean_text.lines().enumerate() {
        let new_line = format_table_line(line, &offsets[linum]);
        new_text.push_str(&new_line);
        new_text.push_str(LINE_END);
    }
    new_text
}

fn contains_table_begin(line: &str) -> bool {
    TABLES_BEGIN.iter().any(|r| line.contains(r))
}

fn contains_table_end(line: &str) -> bool {
    TABLES_END.iter().any(|r| line.contains(r))
}

// Locate all the tables in the lines, as pairs of first and last line,
// leaving out any table with a protected (verbatim or ignored) line
fn find_table_positions(
    lines: &[&str],
    protected: &[bool],
) -> Vec<(usize, usize)> {
    let mut table_positions = vec![];
    let mut begin: Option<usize> = None;
    for (linum, line) in lines.iter().enumerate() {
        if protected.get(linum).copied().unwrap_or(false) {
            begin = None;
        } else if contains_table_begin(line) {
            begin = Some(linum);
        } else if contains_table_end(line) {
            if let Some(b) = begin.take() {
                table_positions.push((b, linum));
            }
        }
    }
    table_positions
}

/// Format all the tables in the text
///
/// `protected` says for each line whether it is verbatim or ignored; a
/// table containing such a line is left alone.
#[must_use]
pub fn format_tables(text: &str, protected: &[bool]) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let table_positions = find_table_positions(&lines, protected);
    if table_positions.is_empty() {
        return text.to_string();
    }
    let mut new_text = String::with_capacity(text.len());
    let mut next = 0;
    for (begin, end) in table_positions {
        for line in &lines[next..begin] {
            new_text.push_str(line);
            new_text.push_str(LINE_END);
        }
        new_text.push_str(&format_table(&lines[begin..=end].join(LINE_END)));
        next = end + 1;
    }
    for line in &lines[next..] {
        new_text.push_str(line);
        new_text.push_str(LINE_END);
    }
    new_text
}

/// Information on the table state of a line
#[derive(Clone, Debug)]
pub struct Table {
    /// Whether the line is in a table
    pub actual: bool,
    /// Whether the line appears to be in a table
    pub visual: bool,
}

impl Table {
    /// Construct a new table state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            actual: false,
            visual: false,
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

// Check if a line is inside a table
#[must_use]
pub fn is_inside_table(line: &str, state: &State, pattern: &Pattern) -> Table {
    let begin = pattern.contains_env_begin && contains_table_begin(line);
    let end = pattern.contains_env_end && contains_table_end(line);
    let actual: bool;
    let visual: bool;
    if begin {
        actual = true;
        visual = true;
    } else if end {
        actual = false;
        visual = true;
    } else {
        actual = state.table.actual;
        visual = state.table.actual;
    }

    Table { actual, visual }
}
