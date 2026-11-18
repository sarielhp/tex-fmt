# Repository Guidelines

## Project Structure & Module Organization

tex-fmt is a LaTeX formatter written in Rust (edition 2021). `src/bin.rs` provides the CLI entry point; `src/lib.rs` exposes the library. Formatting logic lives in focused modules such as `indent.rs`, `wrap.rs`, `table.rs`, and `comments.rs`; argument and configuration handling live in `cli.rs`, `args.rs`, and `config.rs`.

`src/tests.rs` runs fixtures under `tests/`. Each case contains matching `source/` and `target/` files, optionally accompanied by `tex-fmt.toml` or `cli.txt`. `web/` contains the browser interface, `extra/` holds development scripts and graphics, and `man/`, `completion/`, and `ctan/` support distribution and documentation.

## Build, Test, and Development Commands

- `cargo build --release` (or `just build`): build the optimized executable.
- `cargo run -- --print --noconfig path/to/file.tex`: preview formatting locally.
- `cargo test --no-fail-fast` (or `just test`): run the standard test suite.
- `cargo fmt --check`: check Rust formatting, as CI does; use `cargo fmt` to apply it.
- `cargo clippy --release`: run Rust lint checks. `just clippy` also runs `cargo shear` for unused dependencies.
- `cargo doc`: generate documentation for the internals.
- `just wasm`: build the browser package; requires the WASM target, wasm-bindgen, and wasm-opt.

## Coding Style & Naming Conventions

Use rustfmt with the repository's 80-column maximum and default four-space Rust indentation. Follow existing `snake_case` names for modules, functions, and variables, and `PascalCase` for types. The library enables Clippy's pedantic warnings. Keep formatting behavior in the relevant focused module.

## Testing Guidelines

Use Rust's built-in test framework. Add regression fixtures with descriptive directory names and identical filenames under `source/` and `target/`. The harness checks expected output and idempotence. Supply case-specific configuration when needed. No numeric coverage threshold is configured. CI tests stable Rust on Linux, macOS, and Windows.

## Commit & Pull Request Guidelines

Recent commits use short, direct subjects such as `Fix table regex bug`; no mandatory prefix convention is evident. Keep commits focused. Describe the problem, resulting behavior, and validation in pull requests; link relevant issues and include minimal LaTeX examples for formatting changes. Ensure tests and formatting checks pass.

## Configuration Tips

The repository's `tex-fmt.toml` applies automatically when running inside this checkout. Use `--noconfig` or `--config <PATH>` for controlled experiments.

## This Fork (sarielhp/tex-fmt, branch `patched`)

Everything above is upstream's guidance and applies here too. This
section is specific to the fork; keep it at the end of the file so
upstream changes to the sections above merge cleanly.

- The fork carries fixes on top of upstream (see "Changes in this fork"
  in README.md). Its changes are written with AI programming tools
  (Claude Code), directed and reviewed by Sariel Har-Peled.
- Build, test and install into `~/bin/tex-fmt` with `tools/install`.
  The version string is the upstream version plus `+patched`.
- Sync with upstream by fetching the `upstream` remote
  (WGUNDERWOOD/tex-fmt) and merging `upstream/main` into `patched`.
  Never rebase or otherwise rewrite `patched`: it is checked out on
  several hosts and kept in step by `sync_all`, whose pulls fail on
  rewritten history. Expect conflicts mainly in `src/indent.rs`,
  `src/format.rs` and `src/wrap.rs`.
- Branch each upstream contribution from `upstream/main` (upstream pull
  requests target `main`), never from `patched`, keep it to one focused
  change, and disclose in the pull request that it was written with
  Claude Code. Post nothing upstream without the owner's go-ahead.

### Upstream Contributions

Check these periodically:
`gh pr view <number> -R WGUNDERWOOD/tex-fmt --comments` and
`gh pr checks <number> -R WGUNDERWOOD/tex-fmt`. Record every change of
state (review comments, requested changes, CI results, merge or close)
in the log below, newest last, with the date. When a contribution is
merged, drop the fork's own version of the change at the next sync with
upstream.

| PR | Branch | Change | Opened | Status |
|----|--------|--------|--------|--------|
| [#163](https://github.com/WGUNDERWOOD/tex-fmt/pull/163) | `fix-left-right-pairing` | Pair `\left` with `\right` whatever their delimiters (fixes #72, #79, #133) | 2026-11-18 | Open |

Log:

- 2026-11-18 #163 opened against `main`, chosen as the most valuable
  single fix: it resolves upstream's most-reported indentation bug. The
  branch is upstream `main` plus one commit, a minimal version of the
  fork's change (no soft delimiters, no reset at `\end`; `\{` counted as
  before). It adds fixture `tests/left_right`, and `phd_dissertation`'s
  target changes by indentation only. Mergeable; CI had not run yet,
  awaiting the maintainer's approval of workflows for a first-time
  contributor.

Candidates for later pull requests, if #163 is well received (one each):
`\verb` with any delimiter; table fixes (verbatim and `tex-fmt: off`
regions, blank lines, `\&`; char alignment is already upstream #146);
atomic writes; `--args` fail-on-change; the `(:?` regex typo; `\begin`
and `\end` ignored for indentation on lines with `\verb` (a negative
indent after `\begin{center} \verb|x|`). Review
findings for open upstream pull requests #151, #153, #155 and #156 could
be posted as comments there.
