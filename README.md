# tex-fmt <img src="https://github.com/WGUNDERWOOD/tex-fmt/raw/main/extra/logo.svg" alt="tex-fmt" align="right" width=160 />

[![CI](
https://github.com/wgunderwood/tex-fmt/actions/workflows/ci.yml/badge.svg)](
https://github.com/wgunderwood/tex-fmt/actions/workflows/ci.yml)
[![crates.io](
https://img.shields.io/crates/v/tex-fmt?logo=rust)](
https://crates.io/crates/tex-fmt)
[![Packaging status](
https://repology.org/badge/tiny-repos/tex-fmt.svg)](
https://repology.org/project/tex-fmt/versions)
[![license: MIT](
https://shields.io/badge/license-MIT-blue.svg)](
https://mit-license.org/)

> [!NOTE]
> **This is a patched fork of
> [WGUNDERWOOD/tex-fmt](https://github.com/WGUNDERWOOD/tex-fmt),
> developed with AI programming tools:** the changes in this fork were
> written with [Claude Code](https://claude.com/claude-code) (Claude Opus
> 5.5), directed and reviewed by
> [Sariel Har-Peled](https://github.com/sarielhp).
> All credit for tex-fmt itself goes to its author and contributors.
> The fork lives on the `patched` branch; see
> [Changes in this fork](#changes-in-this-fork) for what differs.

## Changes in this fork

Upstream tex-fmt indents by counting `{`, `(` and `[` on each line, so a
single unmatched delimiter shifts every later line of the file. Most of
the changes below make indentation robust against that in real papers;
the rest are bug fixes, several taken from open upstream pull requests
and all followed by an independent code review.

### Indentation

- **`\left` pairs with `\right`** whatever delimiters follow them, so
  `\left\{ ... \right.` and `\left[ ... \right)` are balanced and no longer
  shift the rest of the file
  ([#72](https://github.com/WGUNDERWOOD/tex-fmt/issues/72),
  [#79](https://github.com/WGUNDERWOOD/tex-fmt/issues/79),
  [#133](https://github.com/WGUNDERWOOD/tex-fmt/issues/133)).
- **Escaped braces `\{` and `\}` are soft**: they indent a multi-line set,
  but an unmatched one is forgotten at the next blank line, and a stray
  `\}` is ignored
  ([#124](https://github.com/WGUNDERWOOD/tex-fmt/issues/124)).
- **`\end{name}` resets the indentation** to that of the matching
  `\begin{name}`, so a delimiter left open inside an environment cannot
  shift lines after it.
- **`\verb` contents are skipped** when counting delimiters and
  environments, with any delimiter (`\verb|...|`, `\verb!...!`,
  `\verb*...`).
- **No overflow on deep nesting**: indent levels are 32-bit, so more than
  127 levels (generated TikZ, say) no longer wrap around or panic.
- New option **`indent-delims`** chooses which delimiters indent at all;
  `indent-delims = ["{"]` makes parentheses and brackets not indent, which
  suits math, where half-open intervals such as `$(0,1]$` mix them.
  `(` and `[` must be chosen together for that reason.
- New option **`soft-delims`** makes chosen delimiters soft, like `\{`:
  `soft-delims = ["(", "["]` indents a parenthesis continued onto the next
  line, but forgets it at a blank line.
- New option **`command-blocks`** indents command-delimited blocks such as
  those of `algorithmic`/`algpseudocode`
  ([#107](https://github.com/WGUNDERWOOD/tex-fmt/issues/107)). Each block
  lists its opening command, any middle commands, and its closing
  command; names match exactly, with or without a backslash:

  ```toml
  command-blocks = [
    ["If", "ElsIf", "Else", "EndIf"],
    ["For", "EndFor"],
    ["ForAll", "EndFor"],
    ["While", "EndWhile"],
    ["Repeat", "Until"],
    ["Loop", "EndLoop"],
    ["Procedure", "EndProcedure"],
    ["Function", "EndFunction"],
  ]
  ```

  The opening command indents the following lines, the closing command
  ends that, and a middle command such as `\Else` is dedented on its own
  line, like `\item`. A missing closing command is contained by the reset
  at `\end{algorithmic}`.

All three options default to upstream behaviour (`indent-delims = ["{",
"(", "["]`, no soft delimiters, no command blocks), so an existing
configuration formats as before apart from the fixes above.

### Comments, splitting and wrapping

- **`%` in URLs is not a comment**: inside the URL argument of `\url`,
  `\href` (also with an optional argument), `\nolinkurl` and `\path`,
  including the `\url|...|` form
  ([#55](https://github.com/WGUNDERWOOD/tex-fmt/issues/55)). `\\%` (a line
  break followed by a comment) is now recognized as a comment.
- **`\verb` with any delimiter** is never split or wrapped across lines,
  which LaTeX rejects; upstream knew only `\verb|` and `\verb+`, and only
  the first `\verb` on a line.
- **Wrapping under deep indentation**: a line indented past `wrapmin` is
  wrapped within `wraplen` instead of underflowing (a panic) or being left
  long ([#151](https://github.com/WGUNDERWOOD/tex-fmt/pull/151),
  [#156](https://github.com/WGUNDERWOOD/tex-fmt/pull/156)).
- No false "Line cannot be wrapped" warning on text with accented or Greek
  letters.

### Tables (`--format-tables`)

- Tables inside verbatim environments and `% tex-fmt: off` regions are
  left alone.
- Columns are aligned by character, not byte, so accented text lines up;
  an escaped `\&` no longer shifts the cells after it.
- No blank line is added before a table on the first line or between
  adjacent tables.

### Robustness

- **Files are written atomically**: the result goes to a temporary file
  which then replaces the original, so a full disk or quota can no longer
  leave a file cut short. A failed write is reported, with exit code 1,
  instead of a panic ([#153](https://github.com/WGUNDERWOOD/tex-fmt/pull/153)).
- **Malformed configuration files** are reported, naming the file and the
  field, instead of a panic
  ([#155](https://github.com/WGUNDERWOOD/tex-fmt/pull/155)); invalid
  `command-blocks` are rejected.
- No panics when standard output fails (`--print` into a full disk), on
  file names that are not valid UTF-8, or on files without an extension;
  a file that is not UTF-8 is reported with the reason.
- `--args` shows the right value for `fail-on-change`.

### Installing the fork

```shell
cargo install --git https://github.com/sarielhp/tex-fmt --branch patched
```

or, from a clone, `tools/install`, which runs the tests, builds the
release binary and installs it as `~/bin/tex-fmt`.

An extremely fast LaTeX formatter written in Rust.
Indentation, line wrapping and other formatting for LaTeX source code.
Try it out now in your
[browser](https://wgunderwood.github.io/tex-fmt/)!

<table width="100%">
<tr>
<td>
<b>Input</b>
</td>
<td>
<b>Output</b>
</td>
</tr>
<tr>
<td>

``` tex
\documentclass{article}

\begin{document}

\begin{itemize}
\item Lists with items
over multiple lines
\end{itemize}

\begin{equation}
E = m c^2
\end{equation}

\end{document}
```
</td>
<td>

``` tex
\documentclass{article}

\begin{document}

\begin{itemize}
  \item Lists with items
    over multiple lines
\end{itemize}

\begin{equation}
  E = m c^2
\end{equation}

\end{document}
```
</td>
</tr>
</table>

- ⚡&nbsp; Extremely fast run-time performance
- 🔧&nbsp; Minimal configuration required
- 📟&nbsp; Command line interface
- 📜&nbsp; Handles LaTeX file types `.tex`, `.bib`, `.cls`, and `.sty`
- 🦀&nbsp; Written entirely in safe Rust

## Installation

### Cargo

Install the [stable release](https://crates.io/crates/tex-fmt) with

``` shell
cargo install tex-fmt
```

Install from [GitHub](https://github.com/WGUNDERWOOD/tex-fmt) with

```shell
cargo install --git "https://github.com/wgunderwood/tex-fmt"
```

### Nix

Install from
[nixpkgs](
https://search.nixos.org/packages?channel=unstable&query=tex-fmt)
into a temporary shell with

``` shell
nix-shell -p tex-fmt
```

Build from source using flakes with

``` shell
nix build "github:wgunderwood/tex-fmt"
```

Add to your NixOS installation with

```nix
environment.systemPackages = [
  pkgs.tex-fmt
];
```

It is also included in
[treefmt-nix](https://github.com/numtide/treefmt-nix/tree/main).

### Arch Linux

Install from the
[Arch User Repository](https://aur.archlinux.org/packages/tex-fmt).
For example, using the [yay](https://github.com/Jguer/yay) AUR helper:

``` shell
yay -S tex-fmt
```

### Debian

Install from the [Debian archive](https://www.debian.org/distrib/packages)
(trixie and later):

``` shell
apt install tex-fmt
```

### Homebrew

Install using
[Homebrew](https://formulae.brew.sh/formula/tex-fmt) with

```shell
brew install tex-fmt
```

### Binary download

Binaries for various platforms are available on the GitHub
[releases](https://github.com/WGUNDERWOOD/tex-fmt/releases) page.

### Visual Studio Code

Integration with VS Code is provided by the
[LaTeX Workshop](https://github.com/James-Yu/LaTeX-Workshop)
extension. You will need to first install tex-fmt
through one of the above methods.

### Neovim

A package for Neovim is provided by
[mason.nvim](https://github.com/williamboman/mason.nvim).

### GitHub Action

The [tex-fmt-action](https://github.com/grayespinoza/tex-fmt-action) can install and run tex-fmt.

### CTAN

A package named
[latex-formatter](https://ctan.org/pkg/latex-formatter)
is on CTAN.

## Usage

The most commonly used options are given below.
For a full list, see the
[options](
https://github.com/WGUNDERWOOD/tex-fmt?tab=readme-ov-file#options)
section below.

``` shell
tex-fmt file.tex                   # format file.tex and overwrite
tex-fmt --check file.tex           # check if file.tex is correctly formatted
tex-fmt --print file.tex           # format file.tex and print to stdout
tex-fmt --recursive                # recursively format files in current directory
tex-fmt --recursive dir/           # recursively format files in dir
tex-fmt --fail-on-change file.tex  # format file.tex and return exit-code 1 if overwritten
tex-fmt --nowrap file.tex          # do not wrap long lines
tex-fmt --format-tables file.tex   # format tables (align ampersands)
tex-fmt --stdin                    # read from stdin and print to stdout
tex-fmt --help                     # view help information
```

### Configuration

Options can also be set using a configuration file, which
will be read from the following locations, in order of decreasing priority.

- A named config file passed as `tex-fmt --config <PATH>`
- A file named `tex-fmt.toml` in the current working directory
- A file named `tex-fmt.toml` in the root directory of the current git repository
- A file named `tex-fmt.toml` in a subdirectory titled `tex-fmt/`
  in the user's configuration directory
    - Linux: `~/.config/tex-fmt/tex-fmt.toml`
    - macOS: `/Users/<USER>/Library/Application Support/tex-fmt/tex-fmt.toml`
    - Windows: `C:\Users\<USER>\AppData\Roaming\tex-fmt\tex-fmt.toml`

Arguments passed on the command line will always override those
specified in configuration files. An example configuration file
is available at
[tex-fmt.toml](https://github.com/WGUNDERWOOD/tex-fmt/blob/main/tex-fmt.toml).
To ignore all config files, use the `--noconfig` flag.

Note for contributors: this repository's configuration file will be
automatically applied if tex-fmt is run from within the repository.
Use `--noconfig` or `--config <PATH>` to avoid this.

### Disabling the formatter

Ending a source line with `% tex-fmt: skip` disables formatting for that line.
To disable the formatter for a block, use `% tex-fmt: off` and `% tex-fmt: on`.

``` tex
\documentclass{article}

\begin{document}

    This line is skipped % tex-fmt: skip

% tex-fmt: off
  These lines are also
    not formatted or wrapped
% tex-fmt: on

\end{document}
```

Verbatim environments including `verbatim`, `Verbatim`, `lstlisting`
and `minted` are automatically skipped.

### Ignoring files in recursive mode

Recursive searches with `--recursive` or `-r` will
ignore patterns in `.gitignore` and `.ignore` files,
following git conventions.

### Shell completion

Shell completion scripts can be generated at run-time using the
`--completion <SHELL>` flag. See the
[completion](
https://github.com/WGUNDERWOOD/tex-fmt/tree/main/completion)
directory for more details.

### Man page

A man page can be generated at run-time using the
`--man` flag. See the
[man](https://github.com/WGUNDERWOOD/tex-fmt/tree/main/man)
directory for more details.

### Pre-commit hook

tex-fmt can be run before every git commit using
[pre-commit](http://pre-commit.com) with the following
`.pre-commit-config.yaml` in your repository root:

```yaml
repos:
  - repo: https://github.com/WGUNDERWOOD/tex-fmt
    rev: v0.5.7
    hooks:
      - id: tex-fmt
```

To prevent the pre-commit hook from modifying your files, add:

```yaml
      - id: tex-fmt
        args: [--check]
```

## Performance

When formatting all of the test cases,
tex-fmt is over a thousand times faster than latexindent.

| **Files** | **Lines** | **Size** | **tex-fmt** | **latexindent** | **latexindent -m** |
| --- | --- | --- | --- | --- | --- |
| 51 | 94k | 3.5M | **0.055s** | 106s [x1927] | 127s [x2309] |

## Contribution

Please feel free to open an issue or submit a pull request,
including as much information as you can. Documentation of internals
can be accessed by cloning this repository and running `cargo doc`,
or by visiting the [docs.rs](https://docs.rs/tex-fmt/latest/tex_fmt/) page.

Alternatively, you can become a
[GitHub sponsor](https://github.com/sponsors/WGUNDERWOOD)
or
[Buy Me a Coffee](https://buymeacoffee.com/wgunderwood)!

## Limitations

- Semantic parsing of LaTeX code not conducted
- No linting or correction of syntax errors
- Compliance with existing formatting guidelines not guaranteed
- No spelling or grammar checking

## Existing tools

- [latexindent](https://github.com/cmhughes/latexindent.pl).
Perl script, many configuration options, slow on large files

- [bibtex-tidy](https://github.com/FlamingTempura/bibtex-tidy).
JavaScript program, specifically for BibTeX files

- [LaTeXTidy](http://bfc.sfsu.edu/cgi-bin/hsu.pl?LaTeX_Tidy).
Perl script, download links seem to be broken

- [latex-pretty](https://c.albert-thompson.com/latex-pretty/).
Browser-based, uses latexindent as the backend

- [latexformat.com](https://latexformat.com/).
Browser-based

- [texpretty](http://ftp.math.utah.edu/pub/texpretty/).
C program which works sometimes and appears to be fast

- [latex-editor](https://latex-editor.pages.dev/formatter/).
Browser-based

- [LaTeXFmt](https://github.com/engeljh/vim-latexfmt).
Vim plugin, does not apply indentation

- [latex-formatter](https://github.com/nfode/latex-formatter).
Visual Studio plugin, uses latexindent as the backend

- [LLF](https://repo.or.cz/llf.git).
Lua script, many configuration options

- [Badness](https://badness.dev). 
Rust-based, opinionated, also a language server and linter

## Options

### Command line options

The following arguments can be passed on the command line.

| Option                 | Alias | Default | Description |
| ---------------------- | ----- | ------- | --- |
| `--check`              | `-c`  |         | Check formatting, do not modify files |
| `--print`              | `-p`  |         | Print to stdout, do not modify files |
| `--fail-on-change`     | `-f`  |         | Fail if files are modified |
| `--recursive`          | `-r`  |         | Recursively search for files to format |
| `--nowrap`             | `-n`  |         | Do not wrap long lines |
| `--wraplen <N>`        | `-l`  | `80`    | Line length for wrapping |
| `--tabsize <N>`        | `-t`  | `2`     | Number of characters to use as tab size |
| `--usetabs`            |       |         | Use tabs instead of spaces for indentation |
| `--format-tables`      |       |         | Format tables |
| `--stdin`              | `-s`  |         | Process stdin as a single file, output to stdout |
| `--config <PATH>`      |       |         | Path to config file |
| `--noconfig`           |       |         | Do not read any config file |
| `--verbose`            | `-v`  |         | Show info messages |
| `--quiet`              | `-q`  |         | Hide warning messages |
| `--trace`              |       |         | Show trace messages |
| `--completion <SHELL>` |       |         | Generate a shell completion script |
| `--man`                |       |         | Generate a man page |
| `--args`               |       |         | View arguments passed to tex-fmt |
| `--help`               | `-h`  |         | Print help |
| `--version`            | `-V`  |         | Print version |

### Configuration file options

The following arguments can be provided in `tex-fmt.toml`.
The first example in each row is the default value.

| Option           | Type     | Examples               | Description |
| ---------------- | -------- | ---------------------- | --- |
| `check`          | bool     | `false`                | Check formatting, do not modify files |
| `print`          | bool     | `false`                | Print to stdout, do not modify files |
| `fail-on-change` | bool     | `false`                | Fail if files are modified |
| `wrap`           | bool     | `true`                 | Wrap long lines |
| `wraplen`        | int      | `80`, `100`            | Line length for wrapping |
| `wrapmin`        | int      | `70`, `90`             | Target minimum length for line wrapping |
| `tabsize`        | int      | `2`, `4`               | Number of characters to use as tab size |
| `tabchar`        | str      | `"space"`, `"tab"`     | Character to use for indentation |
| `format-tables`  | bool     | `false`                | Format tables |
| `stdin`          | bool     | `false`                | Process stdin as a single file, output to stdout |
| `lists`          | arr[str] | `[]`, `["myitemize"]`  | Extra list environments to be formatted as `itemize` |
| `verbatims`      | arr[str] | `[]`, `["myverbatim"]` | Extra verbatim environments |
| `no-indent-envs` | arr[str] | `[]`, `["mydocument"]` | Environments which are not indented |
| `wrap-chars`     | arr[str] | `[]`, `["。"]`         | Characters after which lines may be wrapped |
| `indent-delims`  | arr[str] | `["{", "(", "["]`, `["{"]` | Delimiters which affect indentation; `(` and `[` go together |
| `soft-delims`    | arr[str] | `[]`, `["(", "["]`   | Delimiters which are forgotten at a blank line |
| `command-blocks` | arr[arr[str]] | `[]`, `[["If", "Else", "EndIf"]]` | Commands indenting their contents: opening, middle, closing (e.g. `algorithmic`) |
| `verbosity`      | str      | `"warn"`, `"error"`    | Verbosity level for terminal logging |
