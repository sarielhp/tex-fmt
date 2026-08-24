//! Utilities for indenting source lines

use crate::args::Args;
use crate::comments::{find_comment_index, remove_comment};
use crate::format::{Pattern, State};
use crate::logging::{record_line_log, Log};
use crate::regexes::{ENV_BEGIN, ENV_END, ITEM, VERBS};
use core::cmp::max;
use log::Level;
use log::LevelFilter;
use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;

/// Delimiter pairs which may affect indentation, as (opening, closing)
const DELIMS: [(char, char); 3] = [('{', '}'), ('(', ')'), ('[', ']')];

/// Information on the indentation state of a line
#[derive(Debug, Clone)]
pub struct Indent {
    /// The indentation level of a line
    pub actual: i8,
    /// The visual indentation level of a line
    pub visual: i8,
    /// The number of soft delimiters left open in the current paragraph
    pub soft: i8,
    /// The number of delimiters left open overall
    pub depth: i8,
}

impl Indent {
    /// Construct a new indentation state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            actual: 0,
            visual: 0,
            soft: 0,
            depth: 0,
        }
    }
}

impl Default for Indent {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate total indentation change due to the current line
fn get_diff(
    line: &str,
    pattern: &Pattern,
    lists_begin: &[String],
    lists_end: &[String],
    no_indent_envs_begin: &[String],
    no_indent_envs_end: &[String],
) -> i8 {
    // Do not indent if line contains \verb|...|
    if pattern.contains_verb && VERBS.iter().any(|x| line.contains(x)) {
        return 0;
    }

    // Indentation for environments
    let mut diff: i8 = 0;
    if pattern.contains_env_begin && line.contains(ENV_BEGIN) {
        if no_indent_envs_begin.iter().any(|r| line.contains(r)) {
            return 0;
        }
        diff += 1;
        diff += i8::from(lists_begin.iter().any(|r| line.contains(r)));
    } else if pattern.contains_env_end && line.contains(ENV_END) {
        if no_indent_envs_end.iter().any(|r| line.contains(r)) {
            return 0;
        }
        diff -= 1;
        diff -= i8::from(lists_end.iter().any(|r| line.contains(r)));
    }

    diff
}

/// Calculate dedentation for the current line
fn get_back(
    line: &str,
    pattern: &Pattern,
    state: &State,
    lists_end: &[String],
    no_indent_envs_end: &[String],
) -> i8 {
    // Only need to dedent if indentation is present
    if state.indent.actual == 0 {
        return 0;
    }
    let mut back: i8 = 0;

    // Don't apply any indenting if a \verb|...| is present
    if pattern.contains_verb && VERBS.iter().any(|x| line.contains(x)) {
        return 0;
    }

    // Calculate dedentation for environments
    if pattern.contains_env_end && line.contains(ENV_END) {
        // Some environments are not indented
        if no_indent_envs_end.iter().any(|r| line.contains(r)) {
            return 0;
        }
        // List environments get double indents for indenting items
        for r in lists_end {
            if line.contains(r) {
                return 2;
            }
        }
        // Other environments get single indents
        back = 1;
    } else if pattern.contains_item && line.contains(ITEM) {
        // Deindent items to make the rest of item environment appear indented
        back += 1;
    }

    back
}

/// Delimiter depth and soft delimiter count recorded at a `\begin{...}`
#[derive(Debug, Clone)]
pub struct EnvMark {
    /// Name of the environment
    name: String,
    /// Delimiter depth at the `\begin`
    depth: i8,
    /// Number of open soft delimiters at the `\begin`
    soft: i8,
}

/// Running delimiter count for one line
struct DelimCount {
    /// Change in indentation, negative for indenting
    diff: i8,
    /// Largest dedent reached on the line
    back: i8,
    /// Number of soft delimiters open in the current paragraph
    soft: i8,
    /// Number of delimiters open overall
    depth: i8,
}

impl DelimCount {
    fn open(&mut self, is_soft: bool) {
        self.soft += i8::from(is_soft);
        self.depth += 1;
        self.diff -= 1;
    }

    /// A soft closing delimiter without an open partner is ignored
    fn close(&mut self, is_soft: bool) {
        if !is_soft || self.soft > 0 {
            self.soft -= i8::from(is_soft);
            self.depth -= 1;
            self.diff += 1;
        }
    }

    /// Return to the delimiter state recorded at a `\begin`
    fn reset(&mut self, mark: &EnvMark) {
        self.diff += self.depth - mark.depth;
        self.depth = mark.depth;
        self.soft = mark.soft;
    }
}

/// A command which affects delimiter counting
enum Command {
    Left,
    Right,
    Begin(String),
    End(String),
    Other,
}

/// Count the delimiters on a line, in order
///
/// Only delimiter pairs whose opening character is in `args.indent_delims`
/// are counted. Those in `args.soft_delims` are soft: a soft closing
/// delimiter without an open partner is ignored, and open ones are
/// forgotten at a blank line. `\left` and `\right` always count as an
/// opening and a closing delimiter, whichever delimiter follows them, and
/// that delimiter is not counted again, so `\left\{ ... \right.` is
/// balanced. The escaped braces `\{` and `\}` count as braces, but are
/// always soft. At `\end{name}`, the delimiter state returns to the one
/// at the matching `\begin{name}`, since an environment is a group.
fn count_delims(
    line: &str,
    args: &Args,
    count: &mut DelimCount,
    envs: &mut Vec<EnvMark>,
) {
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(b) = chars.next_if(|&n| n == '{' || n == '}') {
                if args.indent_delims.contains(&'{') {
                    if b == '{' {
                        count.open(true);
                    } else {
                        count.close(true);
                    }
                }
            } else {
                match read_command(&mut chars) {
                    Command::Left => count.open(false),
                    Command::Right => count.close(false),
                    Command::Begin(name) => envs.push(EnvMark {
                        name,
                        depth: count.depth,
                        soft: count.soft,
                    }),
                    Command::End(name) => {
                        if let Some(i) =
                            envs.iter().rposition(|m| m.name == name)
                        {
                            count.reset(&envs[i]);
                            envs.truncate(i);
                        }
                    }
                    Command::Other => {}
                }
            }
        } else if let Some(&(open, _)) = DELIMS.iter().find(|&&(o, e)| {
            (o == c || e == c) && args.indent_delims.contains(&o)
        }) {
            let is_soft = args.soft_delims.contains(&open);
            if c == open {
                count.open(is_soft);
            } else {
                count.close(is_soft);
            }
        }
        count.back = max(count.diff, count.back);
    }
}

/// Read the command or escaped character following a backslash
///
/// The delimiter after `\left` or `\right` and the name after `\begin` or
/// `\end` are read as well.
fn read_command(chars: &mut Peekable<Chars>) -> Command {
    let Some(&first) = chars.peek() else {
        return Command::Other;
    };
    if !first.is_ascii_alphabetic() {
        // An escaped character, such as \\ or \%, is skipped, but the math
        // delimiters \( \) \[ \] still count as their bracket
        if !"()[]".contains(first) {
            chars.next();
        }
        return Command::Other;
    }
    let mut name = String::new();
    while let Some(c) = chars.next_if(char::is_ascii_alphabetic) {
        name.push(c);
    }
    match name.as_str() {
        "left" | "right" => {
            skip_delimiter(chars);
            if name == "left" {
                Command::Left
            } else {
                Command::Right
            }
        }
        "begin" | "end" => {
            while chars.next_if(|c| c.is_whitespace()).is_some() {}
            if chars.next_if_eq(&'{').is_none() {
                return Command::Other;
            }
            let mut env = String::new();
            while let Some(c) = chars.next_if(|&c| c != '}') {
                env.push(c);
            }
            chars.next();
            if name == "begin" {
                Command::Begin(env)
            } else {
                Command::End(env)
            }
        }
        "verb" => {
            // \verb|...| or \verb*|...| with any delimiter: skip the
            // contents, which are not LaTeX
            chars.next_if_eq(&'*');
            if let Some(delim) = chars.next() {
                while chars.next_if(|&c| c != delim).is_some() {}
                chars.next();
            }
            Command::Other
        }
        _ => Command::Other,
    }
}

/// Skip the delimiter after `\left` or `\right`: a control word such as
/// `\langle`, a control symbol such as `\{`, or a character such as `(`
fn skip_delimiter(chars: &mut Peekable<Chars>) {
    while chars.next_if(|c| c.is_whitespace()).is_some() {}
    if chars.next() == Some('\\') {
        if chars.next_if(char::is_ascii_alphabetic).is_some() {
            while chars.next_if(char::is_ascii_alphabetic).is_some() {}
        } else {
            chars.next();
        }
    }
}

/// Calculate indentation properties of the current line
#[allow(clippy::too_many_arguments)]
fn get_indent(
    line: &str,
    prev_indent: &Indent,
    pattern: &Pattern,
    state: &State,
    args: &Args,
    envs: &mut Vec<EnvMark>,
    lists_begin: &[String],
    lists_end: &[String],
    no_indent_envs_begin: &[String],
    no_indent_envs_end: &[String],
) -> Indent {
    let mut actual = prev_indent.actual;
    let mut count = DelimCount {
        diff: 0,
        back: 0,
        soft: prev_indent.soft,
        depth: prev_indent.depth,
    };
    // Soft delimiters left open do not carry past a blank line
    if line.trim().is_empty() {
        actual -= count.soft;
        count.depth -= count.soft;
        count.soft = 0;
    }
    let mut diff = get_diff(
        line,
        pattern,
        lists_begin,
        lists_end,
        no_indent_envs_begin,
        no_indent_envs_end,
    );
    let mut back =
        get_back(line, pattern, state, lists_end, no_indent_envs_end);
    count_delims(line, args, &mut count, envs);
    diff -= count.diff;
    back += count.back;
    let visual = actual - back;
    actual += diff;
    Indent {
        actual,
        visual,
        soft: count.soft,
        depth: count.depth,
    }
}

/// Calculates the indent for `line` based on its contents.
/// This functions saves the calculated [Indent], which might be
/// negative, to the given [State], and then ensures that the returned
/// [Indent] is non-negative.
#[allow(clippy::too_many_arguments)]
pub fn calculate_indent(
    line: &str,
    state: &mut State,
    logs: &mut Vec<Log>,
    file: &Path,
    args: &Args,
    pattern: &Pattern,
    lists_begin: &[String],
    lists_end: &[String],
    no_indent_envs_begin: &[String],
    no_indent_envs_end: &[String],
) -> Indent {
    // Calculate the new indent by first removing the comment from the line
    // (if there is one) to ignore diffs from characters in there.
    let comment_index = find_comment_index(line, pattern);
    let line_strip = remove_comment(line, comment_index);
    let mut envs = std::mem::take(&mut state.envs);
    let mut indent = get_indent(
        line_strip,
        &state.indent,
        pattern,
        state,
        args,
        &mut envs,
        lists_begin,
        lists_end,
        no_indent_envs_begin,
        no_indent_envs_end,
    );
    state.envs = envs;

    // Record the indent to the logs.
    if args.verbosity == LevelFilter::Trace {
        record_line_log(
            logs,
            Level::Trace,
            file,
            state.linum_new,
            state.linum_old,
            line,
            &format!(
                "Indent: actual = {}, visual = {}:",
                indent.actual, indent.visual
            ),
        );
    }

    // Save the indent to the state. Note, this indent might be negative;
    // it is saved without correction so that this is
    // not forgotten for the next iterations.
    state.indent = indent.clone();

    // Update the last zero-indented line for use in error messages.
    if indent.visual == 0 && state.linum_new > state.linum_last_zero_indent {
        state.linum_last_zero_indent = state.linum_new;
    }

    // However, we can't negatively indent a line.
    // So we log the negative indent and reset the values to 0.
    if (indent.visual < 0) || (indent.actual < 0) {
        record_line_log(
            logs,
            Level::Warn,
            file,
            state.linum_new,
            state.linum_old,
            line,
            "Indent is negative.",
        );
        indent.actual = indent.actual.max(0);
        indent.visual = indent.visual.max(0);

        // If this is the first negatively indented line, record in the state
        if state.linum_first_negative_indent.is_none() {
            state.linum_first_negative_indent = Some(state.linum_new);
        }
    }

    indent
}

/// Apply the given indentation to a line
#[must_use]
pub fn apply_indent(
    line: &str,
    indent: &Indent,
    args: &Args,
    indent_char: &str,
) -> String {
    let first_non_whitespace = line.chars().position(|c| !c.is_whitespace());

    // If line is blank, return an empty line
    if first_non_whitespace.is_none() {
        return String::new();
    }

    // If line is correctly indented, return it directly
    #[allow(clippy::cast_sign_loss)]
    let n_indent_chars =
        (i16::from(indent.visual) * i16::from(args.tabsize)) as usize;
    if first_non_whitespace == Some(n_indent_chars) {
        return line.into();
    }

    // Otherwise, allocate enough memory to fit line with the added
    // indentation and insert the appropriate string slices
    let trimmed_line = line.trim_start();
    let mut new_line =
        String::with_capacity(trimmed_line.len() + n_indent_chars);
    for idx in 0..n_indent_chars {
        new_line.insert_str(idx, indent_char);
    }
    new_line.insert_str(n_indent_chars, trimmed_line);
    new_line
}
