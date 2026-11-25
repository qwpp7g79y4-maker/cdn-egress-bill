# Quality

Record of 2026-11-25. The checks below ran on commit `c12f04d2c83d30a9a27a537d33c1a2f676af42fd` (`c12f04d`), the tip of `origin/main`, in a clean separate worktree. This commit adds documentation only (this file and `QUALITY.nl.md`).

Toolchain: stable rustc 1.99.0 (b940084d7 2026-09-28) with cargo 1.99.0 (5f94df478 2026-08-27). Host: Linux x86_64. Version 1.0.0 is not published and has no tag.

This page records checks. It is not a guarantee and it is not a fuzz report.

## Commands

Each command ran apart in the project directory, with the flags shown and no others.

| Command | Exit |
| --- | ---: |
| `cargo fmt --check` | 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | 0 |
| `cargo test --locked` | 0 |
| `cargo test --release --locked` | 0 |

## Suite

<!-- SUITE_TESTS: 229 -->

The number is the count of `#[test]` lines under `src/` and `tests/`. Doc-tests are not in it: the README has no Rust examples.

`cargo test --locked` gave 229 passed tests: 123 in the library, 0 in the binary, 1 in `tests/archive_links.rs`, 6 in `tests/claims.rs`, 72 in `tests/cli.rs`, 5 in `tests/docs_clean.rs`, 4 in `tests/documented_shape.rs`, 2 in `tests/examples_golden.rs`, 1 in `tests/handbook_help.rs`, 4 in `tests/json_schema.rs`, 9 in `tests/output.rs`, 1 in `tests/sigint.rs`, 1 in `tests/version.rs`, 0 doc-tests. 0 failed, 0 ignored. `cargo test --release --locked` gave the same counts.

## Not run

This measurement did not run the following; each is not run:

- Rust 1.85 (clippy and tests, debug and release): not run.
- Clippy and `cargo check` for `x86_64-pc-windows-gnu` and `x86_64-apple-darwin`: not run.
- The musl release build and `scripts/check-glibc.sh`: not run.
- `cargo deny`, `cargo audit`: not run.
- `cargo package` (list, build, and the tests in the unpacked package): not run.
- `--offline`, `-j 2` and `--test-threads 2` were not used; the commands ran with the defaults of the host.
- The tables of earlier records (stdout probe, claims per statement, limits, run times) are not repeated here and were not measured again: not run.
- No mutation run, no fuzzing: not run.
