//! Utilities for ignoring verbatim environments

use crate::format::{Pattern, State};
use crate::logging::{record_line_log, Log};
use log::Level::Warn;
use std::path::Path;

/// Byte ranges of the inline verbatim commands on a line
///
/// These are `\verb<d>...<d>` and `\verb*<d>...<d>` with any delimiter
/// `<d>`, and `\mintinline[opts]{lang}<d>...<d>` or `{...}`, where the language
/// may be left out. Each range
/// runs from the backslash to just after the closing delimiter, or to the
/// end of the line if the command is not closed on it.
#[must_use]
pub fn verb_spans(line: &str) -> Vec<(usize, usize)> {
    let mut spans = vec![];
    let mut from = 0;
    while let Some(p) = line[from..].find('\\') {
        let start = from + p;
        let rest = &line[start + 1..];
        let end = if let Some(after) = rest.strip_prefix("verb") {
            let after = after.strip_prefix('*').unwrap_or(after);
            match after.chars().next() {
                Some(d) if !d.is_ascii_alphabetic() && !d.is_whitespace() => {
                    Some(delimited_end(line, line.len() - after.len()))
                }
                _ => None,
            }
        } else {
            rest.strip_prefix("mintinline")
                .map(|after| mintinline_end(line, line.len() - after.len()))
        };
        if let Some(end) = end {
            spans.push((start, end));
            from = end;
        } else {
            // Skip the command name or escaped character, so \\verb is not
            // taken for \verb
            from = start + 1 + rest.chars().next().map_or(0, char::len_utf8);
        }
    }
    spans
}

/// The index after the closing delimiter of a span whose opening
/// delimiter is at `start`, or the end of the line
fn delimited_end(line: &str, start: usize) -> usize {
    let Some(d) = line[start..].chars().next() else {
        return line.len();
    };
    let close = if d == '{' { '}' } else { d };
    let from = start + d.len_utf8();
    line[from..]
        .find(close)
        .map_or(line.len(), |p| from + p + close.len_utf8())
}

/// The end of a `\mintinline` whose options start at `start`
fn mintinline_end(line: &str, mut start: usize) -> usize {
    if line[start..].starts_with('[') {
        match line[start..].find(']') {
            Some(p) => start += p + 1,
            None => return line.len(),
        }
    }
    // The language, if given; tex-fmt also accepts \mintinline|...|
    if line[start..].starts_with('{') {
        match line[start..].find('}') {
            Some(p) => start += p + 1,
            None => return line.len(),
        }
    }
    delimited_end(line, start)
}

/// Information on the verbatim state of a line
#[derive(Clone, Debug)]
pub struct Verbatim {
    /// The verbatim depth of a line
    pub actual: i32,
    /// Whether the line is in a verbatim environment
    pub visual: bool,
}

impl Verbatim {
    /// Construct a new verbatim state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            actual: 0,
            visual: false,
        }
    }
}

impl Default for Verbatim {
    fn default() -> Self {
        Self::new()
    }
}

/// Determine whether a line is in a verbatim environment
#[allow(clippy::too_many_arguments)]
pub fn get_verbatim(
    line: &str,
    state: &State,
    logs: &mut Vec<Log>,
    file: &Path,
    warn: bool,
    pattern: &Pattern,
    verbatims_begin: &[String],
    verbatims_end: &[String],
) -> Verbatim {
    let diff = get_verbatim_diff(line, pattern, verbatims_begin, verbatims_end);
    let actual = state.verbatim.actual + diff;
    let visual = actual > 0 || state.verbatim.actual > 0;

    if warn && (actual < 0) {
        record_line_log(
            logs,
            Warn,
            file,
            state.linum_new,
            state.linum_old,
            line,
            "Verbatim count is negative.",
        );
    }

    Verbatim { actual, visual }
}

/// Calculate total verbatim depth change
fn get_verbatim_diff(
    line: &str,
    pattern: &Pattern,
    verbatims_begin: &[String],
    verbatims_end: &[String],
) -> i32 {
    if pattern.contains_env_begin
        && verbatims_begin.iter().any(|r| line.contains(r))
    {
        1
    } else if pattern.contains_env_end
        && verbatims_end.iter().any(|r| line.contains(r))
    {
        -1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::verb_spans;

    #[test]
    fn finds_verb_with_any_delimiter() {
        assert_eq!(verb_spans(r"a \verb|x| b"), vec![(2, 10)]);
        assert_eq!(verb_spans(r"\verb!a \begin{x}! b"), vec![(0, 18)]);
        assert_eq!(verb_spans(r"\verb*+x y+"), vec![(0, 11)]);
        assert_eq!(verb_spans(r"\verb|x| \verb+y+"), vec![(0, 8), (9, 17)]);
    }

    #[test]
    fn unclosed_verb_runs_to_end_of_line() {
        assert_eq!(verb_spans(r"\verb|x y"), vec![(0, 9)]);
    }

    #[test]
    fn finds_mintinline() {
        assert_eq!(verb_spans(r"\mintinline{py}|x| y"), vec![(0, 18)]);
        assert_eq!(verb_spans(r"\mintinline[o]{py}{x} y"), vec![(0, 21)]);
        assert_eq!(verb_spans(r"\mintinline|x| y"), vec![(0, 14)]);
    }

    #[test]
    fn ignores_other_commands() {
        assert_eq!(verb_spans(r"\verbatim \verbx \\verb|x|"), vec![]);
        assert_eq!(verb_spans(r"\begin{verbatim}"), vec![]);
    }
}
