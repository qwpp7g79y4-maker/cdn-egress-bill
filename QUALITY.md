# Quality

pep: 10

Record of 2026-11-25. The checks below ran on commit `c9f1f66cedb97bd7a037a96c97febfee8168d4fc` (`c9f1f66`), the tip of the branch `stdout-probe`, in a clean worktree. The commit that records this page changes documentation only (this file and `QUALITY.nl.md`).

Toolchain: stable rustc 1.99.0 (b940084d7 2026-09-28) with cargo 1.99.0 (5f94df478 2026-08-27). Host: Linux x86_64. Version 1.0.0 has no tag.

This page records checks. It is not a guarantee and it is not a fuzz report.

## Source of the prices

The tool reproduces the AWS CloudFront pay-as-you-go list prices (Data Transfer Out to Internet and HTTP(S) requests). The documentation it copies is at https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/ . The snapshot in `data/prices.txt` was checked on 2026-11-09. This measurement did not fetch that page again.

## Commands

Each command ran apart in the project directory, with the flags shown and no others.

| Command | Exit |
| --- | ---: |
| `cargo fmt --check` | 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | 0 |
| `cargo test --locked` | 0 |
| `cargo test --release --locked` | 0 |

## Rust 1.85

Added 2026-11-25. The four commands below ran with `cargo +1.85` (cargo 1.85.1 (d73d2caf9 2024-12-31)), on branch `stdout-probe` at commit `c9f1f66`, with no code changed since the commit named at the top. Each ran apart, with the flags shown and no others.

| Command | Exit |
| --- | ---: |
| `cargo +1.85 fmt --check` | 0 |
| `cargo +1.85 clippy --all-targets --locked -- -D warnings` | 0 |
| `cargo +1.85 test --locked` | 0 |
| `cargo +1.85 test --release --locked` | 0 |

`cargo +1.85 test --locked` gave 229 passed tests, with the same counts per file as below. 0 failed, 0 ignored. `cargo +1.85 test --release --locked` gave 229 passed tests, 0 failed, 0 ignored (2026-11-25).

## Suite

<!-- SUITE_TESTS: 229 -->

The number is the count of `#[test]` lines under `src/` and `tests/`. Doc-tests are not in it: the README has no Rust examples.

`cargo test --locked` gave 229 passed tests: 123 in the library, 0 in the binary, 1 in `tests/archive_links.rs`, 6 in `tests/claims.rs`, 72 in `tests/cli.rs`, 5 in `tests/docs_clean.rs`, 4 in `tests/documented_shape.rs`, 2 in `tests/examples_golden.rs`, 1 in `tests/handbook_help.rs`, 4 in `tests/json_schema.rs`, 9 in `tests/output.rs`, 1 in `tests/sigint.rs`, 1 in `tests/version.rs`, 0 doc-tests. 0 failed, 0 ignored. `cargo test --release --locked` gave the same counts.

## Not run

This measurement did not run the following; each is not run:

- Clippy and `cargo check` for `x86_64-pc-windows-gnu` and `x86_64-apple-darwin`: not run.
- The musl release build and `scripts/check-glibc.sh`: not run.
- `cargo deny`, `cargo audit`: not run.
- `cargo package` (list, build, and the tests in the unpacked package): not run.
- `--offline`, `-j 2` and `--test-threads 2` were not used; the commands ran with the defaults of the host.
- The tables of earlier records (stdout probe, claims per statement, limits, run times) are not repeated here and were not measured again: not run.
- No mutation run, no fuzzing: not run.
