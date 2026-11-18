//! Utilities for finding, extracting and removing LaTeX comments

use crate::format::Pattern;

/// Commands whose first argument is a URL, in which `%` is not a comment
const URL_COMMANDS: [&str; 4] = ["url", "href", "nolinkurl", "path"];

/// Find the location where a comment begins in a line
///
/// A `%` starts a comment unless it is escaped as `\%`, it is inside
/// `\verb|...|`, or it is inside the URL argument of `\url`, `\href`,
/// `\nolinkurl` or `\path`, where raw percent signs are allowed (#55). The
/// URL argument may follow spaces and, for `\href`, an optional argument;
/// `\url` and `\path` also take the form `\url|...|`. A `%` after `\\`
/// does start a comment. A URL continued from the previous line is not
/// recognized.
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
            b'\\' => i = skip_command(line, i + 1),
            _ => i += 1,
        }
    }
    None
}

/// The index after the command whose name starts at `start`, just after a
/// backslash, including the contents of `\verb` and of URL arguments
///
/// Indices are byte indices. Only ASCII bytes are examined, so an index
/// inside a multi-byte character is harmless: scanning continues to the
/// next ASCII byte.
fn skip_command(line: &str, start: usize) -> usize {
    let bytes = line.as_bytes();
    let end = start
        + bytes[start..]
            .iter()
            .take_while(|b| b.is_ascii_alphabetic())
            .count();
    if end == start {
        // An escaped character, such as \% or \\
        return start + 1;
    }
    let name = &line[start..end];
    if name == "verb" {
        let after_star = end + usize::from(bytes.get(end) == Some(&b'*'));
        return skip_to_delimiter(line, after_star);
    }
    if !URL_COMMANDS.contains(&name) {
        return end;
    }
    let mut i = skip_spaces(bytes, end);
    if name == "href" && bytes.get(i) == Some(&b'[') {
        i = bytes[i..]
            .iter()
            .position(|&b| b == b']')
            .map_or(bytes.len(), |p| skip_spaces(bytes, i + p + 1));
    }
    match bytes.get(i) {
        Some(b'{') => skip_group(bytes, i + 1),
        Some(_) if name == "url" || name == "path" => {
            skip_to_delimiter(line, i)
        }
        _ => i,
    }
}

/// The index after the next occurrence of the delimiter at `start`, as in
/// `\verb|...|`, or the end of the line
fn skip_to_delimiter(line: &str, start: usize) -> usize {
    let Some(delim) = line.get(start..).and_then(|s| s.chars().next()) else {
        return line.len();
    };
    let from = start + delim.len_utf8();
    line[from..]
        .find(delim)
        .map_or(line.len(), |p| from + p + delim.len_utf8())
}

/// The index of the first byte at or after `start` which is not a space
fn skip_spaces(bytes: &[u8], start: usize) -> usize {
    start
        + bytes[start..]
            .iter()
            .take_while(|b| b.is_ascii_whitespace())
            .count()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn comment(line: &str) -> Option<usize> {
        find_comment_index(line, &Pattern::new(line))
    }

    #[test]
    fn finds_comments_and_skips_escapes() {
        assert_eq!(comment("a % b"), Some(2));
        assert_eq!(comment(r"50\% off"), None);
        assert_eq!(comment(r"line\\% comment"), Some(6));
        assert_eq!(comment(r"a\\\% b"), None);
        assert_eq!(comment("\\\u{e9}% b"), Some(3));
    }

    #[test]
    fn skips_url_arguments() {
        assert_eq!(comment(r"\url{a%20b} % c"), Some(12));
        assert_eq!(comment(r"\url{a%20b{c}d}"), None);
        assert_eq!(comment(r"\href{a%20b}{x} % c"), Some(16));
        assert_eq!(comment(r"\href[opt]{a%20b}{x}"), None);
        assert_eq!(comment(r"\url {a%20b}"), None);
        assert_eq!(comment(r"\url|a%20b| % c"), Some(12));
        assert_eq!(comment(r"\path{a%b}"), None);
        assert_eq!(comment(r"\nolinkurl{a%b}"), None);
    }

    #[test]
    fn other_commands_do_not_hide_comments() {
        assert_eq!(comment(r"\urlstyle{x%y}"), Some(11));
        assert_eq!(comment(r"\pathname{a%b}"), Some(11));
        assert_eq!(comment(r"\href{a}{50%}"), Some(11));
    }

    #[test]
    fn skips_verb_contents() {
        assert_eq!(comment(r"\verb|50%| % c"), Some(11));
        assert_eq!(comment(r"\verb*!%! x"), None);
        assert_eq!(comment(r"\verb|\url{| % c"), Some(13));
        assert_eq!(comment(r"\verb|unclosed %"), None);
    }

    #[test]
    fn unclosed_url_runs_to_end_of_line() {
        assert_eq!(comment(r"\url{a%b"), None);
    }
}
