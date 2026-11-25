# Quality

This commit changes documentation only; the measurements below apply to code commit 4a90b3f.

Record of 2026-11-20, extended on 2026-11-24. Toolchains: stable rustc 1.99.0 (b940084d7 2026-09-28) with cargo 1.99.0 (5f94df478 2026-08-27), and rustc 1.85.1 (4eb161250 2025-03-15) with cargo 1.85.1 (d73d2caf9 2024-12-31). Host: Linux x86_64. Branch `poets-cdn`. The checks below ran on the code of commit `4a90b3f0b5e4edfc291a05aad9d1d69a21659ddb` (`4a90b3f`): `cargo fmt --check`, both clippy runs (stable and 1.85), `cargo test` (debug and release, stable and 1.85, 229 passed) and the package list. The other rows of the Commands table (cross-target clippy and check, musl build, glibc check, deny, audit, package and the test in the unpacked package) ran on `591915f` on 2026-11-20 and were not run again on `4a90b3f`: not run. Commit `4a90b3f` changes `.github/workflows/release.yml` and adds `tests/archive_links.rs`; the commits after it change documentation only. Version 1.0.0 is not published and has no tag.

This page records checks. It is not a guarantee and it is not a fuzz report.

## Source of the log format

The input shape tested in `tests/documented_shape.rs` (33 fields, `-` for no value, percent-encoded paths, a header without rows) is the CloudFront standard (legacy) access log as documented at https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/standard-logging-legacy-s3.html .

## Suite

<!-- SUITE_TESTS: 229 -->

The number is the count of `#[test]` lines under `src/` and `tests/`. Doc-tests are not in it: the README has no Rust examples.

`cargo test --locked` gave 229 passed tests: 123 in the library, 0 in the binary, 1 in `tests/archive_links.rs`, 6 in `tests/claims.rs`, 72 in `tests/cli.rs`, 5 in `tests/docs_clean.rs`, 4 in `tests/documented_shape.rs`, 1 in `tests/handbook_help.rs`, 9 in `tests/output.rs`, 4 in `tests/json_schema.rs`, 2 in `tests/examples_golden.rs`, 1 in `tests/sigint.rs`, 1 in `tests/version.rs`, 0 doc-tests. 0 failed, 0 ignored. Debug, release, Rust 1.85 (debug) and Rust 1.85 (release) give the same counts. The count before the gzip member limit was 183; the six new tests are named under Red before green.

## Commands

Each command ran apart in the project directory, with two compile jobs (`-j 2`) and `--test-threads 2` for tests. The commands that use `--locked --offline` ran with both flags.

| Command | Exit |
| --- | ---: |
| `cargo fmt --check` | 0 |
| `cargo clippy --locked --offline --all-targets -- -D warnings` | 0 |
| `cargo +1.85 clippy --locked --offline --all-targets -- -D warnings` | 0 |
| `cargo clippy --locked --offline --all-targets --target x86_64-pc-windows-gnu -- -D warnings` | 0 |
| `cargo clippy --locked --offline --all-targets --target x86_64-apple-darwin -- -D warnings` | 0 |
| `cargo check --locked --offline --tests --target x86_64-pc-windows-gnu` | 0 |
| `cargo check --locked --offline --tests --target x86_64-apple-darwin` | 0 |
| `cargo build --locked --offline --release --target x86_64-unknown-linux-musl` | 0 |
| `cargo test --locked --offline --no-fail-fast -- --test-threads 2` | 0 |
| `cargo test --locked --offline --no-fail-fast --release -- --test-threads 2` | 0 |
| `cargo +1.85 test --locked --offline --no-fail-fast -- --test-threads 2` | 0 |
| `cargo +1.85 test --locked --offline --no-fail-fast --release -- --test-threads 2` | 0 |
| `cargo build --locked --offline --release`, then `bash scripts/check-glibc.sh target/release/cdn-egress-bill 2.34` (highest symbol GLIBC_2.34) | 0 |
| `cargo deny --offline check licenses` | 0 |
| `cargo audit --no-fetch` (29 crates, 1527 advisories) | 0 |
| `cargo package --locked --offline --allow-dirty --no-verify --list` (52 paths) | 0 |
| `cargo package --locked --offline --allow-dirty --no-verify` | 0 |
| `cargo test --offline --no-fail-fast -- --test-threads 2` in the unpacked `.crate` made by `cargo package` (see below) | 0 |

The internal-name check is `tests/docs_clean.rs`, part of the suite above.

The two `--target` checks (and the two clippy runs for those targets) only compile for Windows and macOS; nothing ran there. They ran with the stable toolchain; the targets are not installed for Rust 1.85, so clippy 1.85 for them was not run. `cargo deny` printed no warning and `cargo audit` scanned 29 crates against 1527 advisories (advisory database as it was on this host, no fetch).

The package was also unpacked into an empty directory and tested there: `cargo test --locked --offline --no-fail-fast` in the unpacked crate passed with 228 tests (123, 0, 6, 72, 5, 4, 2, 1, 4, 9, 1, 1, 0) on `591915f`; it was not run again on `4a90b3f`, which adds `tests/archive_links.rs` (it returns at once there, because `.github/` is not in the package), because the second check of `tests/docs_clean.rs` skips a path that is not in the package (`.github/`, `.gitattributes`, `scripts/`) and only demands a folder's main file when the folder is there.

`cargo package --list` is the package list (52 paths, with `.gitignore` and `.cargo_vcs_info.json` in a git checkout). It holds `man/cdn-egress-bill.1`, `SECURITY.md`, `examples/`, `build.rs`, `about.hbs`, `about.toml`, `README.nl.md`, `CONTRIBUTING.md`, `src/interrupt.rs`, `src/stdout_probe.rs`, `tests/archive_links.rs` and `tests/docs_clean.rs`. It has no `.github/`, no `.gitattributes`, no `scripts/` and no working notes. The two greps for internal names and working-note file names (`git grep -n -i --untracked`, with the word lists of `tests/docs_clean.rs` and the two file names, the second one without `.gitignore` and `Cargo.toml`) gave no hit; `.gitignore` holds the two working-note file names on purpose, so that they stay out of git.

## Test result per target

Debug (`cargo test --locked --offline --no-fail-fast -- --test-threads 2`):

| Target | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| unittests `src/lib.rs` | 123 | 0 | 0 |
| unittests `src/main.rs` | 0 | 0 | 0 |
| `tests/archive_links.rs` | 1 | 0 | 0 |
| `tests/claims.rs` | 6 | 0 | 0 |
| `tests/cli.rs` | 72 | 0 | 0 |
| `tests/docs_clean.rs` | 5 | 0 | 0 |
| `tests/documented_shape.rs` | 4 | 0 | 0 |
| `tests/examples_golden.rs` | 2 | 0 | 0 |
| `tests/handbook_help.rs` | 1 | 0 | 0 |
| `tests/json_schema.rs` | 4 | 0 | 0 |
| `tests/output.rs` | 9 | 0 | 0 |
| `tests/sigint.rs` | 1 | 0 | 0 |
| `tests/version.rs` | 1 | 0 | 0 |
| doc-tests | 0 | 0 | 0 |
| total | 229 | 0 | 0 |

Release (`cargo test --locked --offline --no-fail-fast --release -- --test-threads 2`):

| Target | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| unittests `src/lib.rs` | 123 | 0 | 0 |
| unittests `src/main.rs` | 0 | 0 | 0 |
| `tests/archive_links.rs` | 1 | 0 | 0 |
| `tests/claims.rs` | 6 | 0 | 0 |
| `tests/cli.rs` | 72 | 0 | 0 |
| `tests/docs_clean.rs` | 5 | 0 | 0 |
| `tests/documented_shape.rs` | 4 | 0 | 0 |
| `tests/examples_golden.rs` | 2 | 0 | 0 |
| `tests/handbook_help.rs` | 1 | 0 | 0 |
| `tests/json_schema.rs` | 4 | 0 | 0 |
| `tests/output.rs` | 9 | 0 | 0 |
| `tests/sigint.rs` | 1 | 0 | 0 |
| `tests/version.rs` | 1 | 0 | 0 |
| doc-tests | 0 | 0 | 0 |
| total | 229 | 0 | 0 |

Rust 1.85 debug (`cargo +1.85 test ...`) and Rust 1.85 release (`cargo +1.85 test ... --release`) gave the same per-target numbers and total 229.

## Red before green

Each behaviour change since commit 7670d48 has a test that failed on the code of that commit and passes after the change. The tests were run on the old code before `src/` was changed (and again on a copy of commit 7670d48 with the new test files):

| Test | Before the change |
| --- | --- |
| `a_fields_line_with_many_names_is_read_in_linear_time` | failed: `a #Fields line of 200000 names must not take 20 seconds`. The test stops the binary after 20 seconds. A `#Fields:` line of 100000 names took 13.4 seconds with the release binary |
| `a_price_file_with_many_regions_and_labels_is_read_in_linear_time` | failed: `100000 region lines must not take 20 seconds`. A price file with 40000 extra regions took 2.2 seconds with the release binary |
| `a_gzip_file_of_more_members_than_the_maximum_is_refused` | failed on the code of commit 242d332: `assertion `left == right` failed: empty`, left `Some(0)`, right `Some(2)` (the file of 500001 members was read to the end, exit 0). The same commit took 3.9 seconds for 1000000 empty members, 15.5 seconds for 4000000 and 53 seconds for 13421700 members (268434080 bytes) with the release binary |

The four tests of the gzip member limit that read 200000 members, or a damaged member among many, are characterisations: they pass on the old code as well (200000 empty members took 0.8 seconds there, under the 5 seconds they promise), and they guard the behaviour that stayed the same (a BOM, a name, a comment or an extra field per member, the same message for a damaged member). Only the member-count test is red on the old code. Without optimised dependencies the debug binary needed 5.7 to 6.2 seconds for 200000 members, so `Cargo.toml` builds the dependencies with `opt-level = 3` in the dev profile.

After the change both pass in under a second in a debug build (the whole of `tests/cli.rs` takes 9 seconds since the gzip member tests were added). The price step of a month (all paths once per region) has the same cause and no test of its own: a log of 200000 paths with a price file of 10000 regions took 118 seconds before and 0.5 seconds after, with the same output (text and JSON compared byte for byte on four logs, and with a price file of 10000 regions).

The sentences added to the README and the man page about usage errors are checked by `help_long_options_match_the_man_page_and_unknown_option_is_not_a_panic`, which failed on the old man page and passes now.

These tests describe behaviour that was already there. They passed on the code before this record, so they are not proof of a fix; they pin the behaviour so that a change shows up. For each of them a change was made on a copy of the project in a temporary directory, the test was seen to fail, and the project itself was not touched:

- `a_plain_file_with_an_upper_case_gz_name_is_refused_as_gzip`: with the test for the `.gz` name in `src/parse.rs` made case sensitive, it fails (`gzip_is_recognised_by_its_first_bytes_and_by_an_upper_case_name` stays green, because a real gzip file is also found by its first bytes);
- the handbook check: with `--strict` taken out of the OPTIONS section of the man page, the test fails on the comparison of long options; with `--strict` changed in the whole man page, it fails on the verbatim `--help` text;
- the docs check: with README lines that hold a local absolute path, a home-relative path, a wrapper option and the names of five internal tools, `public_docs_have_no_internal_names_or_local_paths` fails and names each line and word; with a local path in the man page it fails on the man page; with `.github` taken out of the list of the second check, `package_metadata_workflows_sources_and_tests_have_no_internal_names_either` fails (`.github/ exists but ci.yml was not read`).

The earlier changes (commits 578cd2f, d5c179e and 7670d48) have their own tests, among them `clap_error_with_a_non_utf8_argument_and_a_line_break_stays_on_one_line`, `clap_error_with_an_escape_sequence_and_a_line_break_stays_on_one_line`, `clap_error_ends_with_one_newline_and_no_blank_line`, `gzip_members_may_split_a_utf8_character` and `non_utf8_month_value_keeps_clap_lines_and_non_utf8_option_shows_its_bytes` (red on the code of commit d5c179e at its `!err.contains("\\u000a")` assert). Other tests from those records (`a_larger_remainder_beats_a_larger_weight`, `leap_days_and_month_numbers_follow_the_calendar`, `a_single_trailing_tab_on_a_data_row_is_tolerated`, `a_file_of_exactly_the_size_limit_is_read_and_one_byte_more_is_refused`, `the_order_of_the_log_lines_does_not_change_the_report`, `quick_start_log_and_report_match_the_examples_and_the_binary`, `dates_and_months_are_accepted_from_1970_to_2100_only`, `sc_bytes_is_an_unsigned_64_bit_number_and_a_sum_that_does_not_fit_is_an_error`, `an_empty_log_and_a_header_only_log_succeed_even_with_strict`, `a_hash_line_that_holds_a_tab_is_a_data_row`) are pins in the same sense.

The CI workflow, `scripts/check-glibc.sh` (run by hand against the release binary here: highest symbol 2.34, exit 0) and the dependency policy comment have no test; they were compared by eye with a sister project and are listed under Known limits.

Changes of the session of 2026-11-20 (each test was run on the code before the change and failed there):

| Test | Before the change |
| --- | --- |
| `sigint_while_the_report_is_blocked_on_a_full_pipe_exits_130` | failed: `ExitStatus(unix_wait_status(2))`, code `None` instead of `Some(130)` (killed by the signal, no message) |
| `tests/output.rs` | 6 of 9 failed: `error: unexpected argument '--output' found` (the 3 that passed on the old code only check an exit 2 or a kept file) |
| `tests/json_schema.rs` | 3 of 4 failed: the JSON started with `"estimate": true` and had no `schema_version` (the fourth checks that an error prints no JSON) |
| `version_is_the_crate_version_and_in_a_debug_checkout_also_the_commit` | failed: left `"cdn-egress-bill 1.0.0\n"`, right `"cdn-egress-bill 1.0.0 (e5c1ebb)\n"` |
| `tests/examples_golden.rs` | 2 of 2 failed: `examples/access.report.md` and `examples/README.md` did not exist |

Change of 2026-11-24 (code commit `4a90b3f`): the release archive was not self-contained. The `Package` step of `.github/workflows/release.yml` did not copy `QUALITY.nl.md` (linked from `README.nl.md`), `SECURITY.md` (named in both READMEs) or `data/` (`data/prices.txt` and `data/edges.txt`, named in both READMEs). The test `every_relative_link_in_the_packaged_documents_points_into_the_archive` in `tests/archive_links.rs` reads the `cp` lines of that step and follows every relative link in the documents it copies, and the top-level `.md` files and `data/` files that the READMEs name. It was run on the workflow of commit 87da8e3 (the new test file with the old `release.yml`) and failed:

```
---- every_relative_link_in_the_packaged_documents_points_into_the_archive stdout ----

thread 'every_relative_link_in_the_packaged_documents_points_into_the_archive' (…) panicked at tests/archive_links.rs:135:5:
not in the release archive: [
    "README.md -> SECURITY.md",
    "README.md -> data/edges.txt",
    "README.md -> data/prices.txt",
    "README.nl.md -> QUALITY.nl.md",
    "README.nl.md -> SECURITY.md",
    "README.nl.md -> data/edges.txt",
    "README.nl.md -> data/prices.txt",
]

failures:
    every_relative_link_in_the_packaged_documents_points_into_the_archive

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

After the change to `release.yml` it passes. In the commit that adds it, `quality_and_readme_share_suite_count` in `tests/claims.rs` fails (229 tests counted, 228 stated); the next commit, which changes documentation only, states 229 and it passes again. The workflow also has a new smoke step that checks the same links in the unpacked archive; it was run by hand with the same shell lines on a copy of the files made with the new `cp` lines (exit 0), and with `SECURITY.md` removed from the copy (exit 1, `missing: README.md -> SECURITY.md`). GitHub did not run it: not run.

## The stdout probe

`src/stdout_probe.rs` holds the only `unsafe` in the crate. Rust replaces a closed file descriptor 0, 1 or 2 with `/dev/null` (read-write) before `main` runs, so a really closed stdout could not be told from `1<>/dev/null` afterwards. On Linux, a constructor in `.init_array` runs earlier and records with a read-only `fcntl(1, F_GETFD)` whether descriptor 1 was already closed. `src/lib.rs` declares the module (`mod stdout_probe`, the only `#[allow(unsafe_code)]`) and exports `stdout_was_closed_at_start`; `src/main.rs` calls it in `write_stdout` on Linux and returns the stdout error (exit 2) when it is true. On another unix system `main.rs` uses its older check instead. The file is part of the commit.

## Closed and full stdout

Measured on 2026-11-19 with the release binary built from this tree and with the debug binary, for every row, with the log of the README quick start (`examples/access.log`). Exit code and message of each run:

| Command (`BIN` is the binary, `LOG` is `examples/access.log`) | Exit | Message on stderr |
| --- | ---: | --- |
| `BIN LOG >&-` | 2 | `stdout: Bad file descriptor (os error 9)` |
| `BIN LOG >/dev/full` | 2 | `stdout: No space left on device (os error 28)` |
| `BIN LOG >/dev/null` | 0 | none |
| `BIN LOG 1<>/dev/null` | 0 | none |
| `python3 -I -c '... subprocess.run([BIN, LOG], stdout=subprocess.DEVNULL) ...'` | 0 | none |
| `BIN LOG \| true` | 2 | `stdout: Broken pipe (os error 32)` |
| `BIN --help >&-` | 2 | `stdout: Bad file descriptor (os error 9)` |
| `BIN --version >&-` | 2 | `stdout: Bad file descriptor (os error 9)` |
| `BIN --help >/dev/full` | 2 | `stdout: No space left on device (os error 28)` |
| `BIN --version >/dev/full` | 2 | `stdout: No space left on device (os error 28)` |
| `BIN --help >/dev/null` | 0 | none |
| `BIN --version 1<>/dev/null` | 0 | none |
| `python3 -I` with `stdout=subprocess.DEVNULL` for `--help` and `--version` | 0 | none |
| `BIN --strict LOG >&-` | 2 | `stdout: Bad file descriptor (os error 9)` |
| `BIN --format json LOG >/dev/full` | 2 | `stdout: No space left on device (os error 28)` |
| in a mount namespace with an empty `/proc`: `BIN LOG > file` | 0 | none (788 bytes in the file) |
| in the same namespace: `BIN LOG 1<>/dev/null` | 0 | none |
| in the same namespace: `BIN LOG >&-` | 2 | `stdout: Bad file descriptor (os error 9)` |

The musl build (`x86_64-unknown-linux-musl`, release) gave the same exit codes and messages for `>&-`, `>/dev/full` and `1<>/dev/null`. No run panicked. Thirty runs of `BIN LOG | true` all gave exit 2, with the debug binary and with the release binary.

## Usage errors with escape sequences

Measured on 2026-11-19 with the release and the debug binary, with the same result. Each is one block of clap's own lines, with the argument written as `\u....`, or as `\xff` and `\n` when it is not UTF-8:

| Command | Exit | stderr |
| --- | ---: | --- |
| `BIN --format $'1\x1b[31m\nFAKE: line' LOG` | 2 | `error: invalid value '1\u001b[31m\u000aFAKE: line' for '--format <FORMAT>'`, then clap's own `[possible values: text, json]` line |
| `BIN $'--x\x1b[31m\nFAKE: line' LOG` | 2 | `error: unexpected argument '--x\u001b[31m\u000aFAKE: line' found`, the tip line with the same text, and the usage |
| `BIN $'--x\x1b[31m\rFAKE: line\xe2\x80\xae' LOG` | 2 | `\u000d` and `\u202e` in place of the CR and the bidi character, no line of its own |
| `BIN $'--x\xff\nFAKE: line' LOG` | 2 | `error: unexpected argument '--x\xff\nFAKE: line' found`, on one line |
| `BIN --format $'\xff\x1b[31m\nFAKE: line' LOG` | 2 | `error: invalid value '\xff\x1b[31m\nFAKE: line' for '--format <FORMAT>'`, then clap's own `[possible values: text, json]` line |
| `BIN --month $'\xff' LOG` | 2 | clap's own message in its own lines: `error: invalid UTF-8 was detected in one or more arguments`, a blank line, the usage line, a blank line, `For more information, try '--help'.` |
| `BIN --prices $'1\x1b[31m\nFAKE: line' LOG` (the same for `--edges`) | 2 | one line: `1\u001b[31m\u000aFAKE: line: No such file or directory (os error 2)` |
| `BIN --month $'1\x1b[31m\nFAKE: line' LOG` | 2 | one line: `maand 1\u001b[31m\u000aFAKE: line is ongeldig` |
| `BIN $'-\xff\nFAKE' LOG` | 2 | `error: unexpected argument '-\' found`, no raw line break (see Known limits) |
| `BIN` (no arguments) | 2 | clap's text, ending with `try '--help'.` and one newline |

## Not run

- A fuzz campaign.
- A mutation run. The few changes made on a copy are listed under Red before green; they were made to see one test fail, not to measure the strength of the suite.
- GitHub Actions (the CI file and `.github/workflows/release.yml`). The workflows were not started (GitHub Actions cannot run for this account at the moment because of a payment problem), so no run of either file was seen. The release workflow was only parsed as YAML, so the changed `cp` lines and the new link check in the smoke step have not run on GitHub: not run.
- `cargo about generate about.hbs`: `cargo-about` is not installed on this host, so `THIRD-PARTY-LICENSES.html` was not generated and `about.toml` was not run. Its list of accepted licenses is the list in `deny.toml`, and `cargo deny check licenses` passed.
- The tests on the musl build. Only the musl release build, the quick start report and the three stdout rows named above were run on it.
- Clippy 1.85 for the Windows and macOS targets (not installed for that toolchain).
- Windows and macOS. The code compiles for `x86_64-pc-windows-gnu` and `x86_64-apple-darwin` (`cargo check --tests` and clippy), but no test ran there. Tests that need `/dev/full`, `/proc`, a closed file descriptor or a name that is not UTF-8 are gated to Linux, and the non-Linux stdout check in `src/main.rs` was not run.
- Real CloudFront logs; only the synthetic fixtures and logs made for the tests.
- Live AWS prices. The rates are the table in `data/prices.txt` as checked on 2026-11-09.
- `git commit`, `git push`, `git tag`, and `cargo publish`.

## Known limits

- This page records checks. It is not a guarantee and it is not a fuzz report.
- Memory. A log is read completely into memory. A text log of 50000093 bytes with 1345030 different paths took 1.2 GB at peak and 3.6 seconds (about 24 times its size). The limit of 268435456 bytes is on the file as stored. A gzip file is not limited after unpacking: a gzip of 1 GiB of zero bytes (4.7 MB) was unpacked completely, 2.1 GB at peak, before the run ended with exit 2 (`geen #Fields vóór de data`). Memory use grows with the unpacked size (about twice it in this measurement), so a gzip file that unpacks to many gigabytes needs that much memory and was not measured further. Bounding the unpacked size is a product decision and is open (see Open points).
- Gzip members. Every member sets up a decoder (about 4 microseconds, also for an empty member or a member with only a BOM). Measured with the release binary, 1x, 2x and 4x the input: 124999, 249999 and 499999 empty members after a header member took 0.44, 0.89 and 1.84 seconds (5.0, 7.1 and 12.2 MB at peak), the same with a BOM in each member 0.46, 0.93 and 1.82 seconds. A file of 500001 members or more is refused with exit 2 after reading 500001 members: 1000000, 4000000 and 13421700 members (268434080 bytes) took 1.85, 1.89 and 1.98 seconds (22, 81 and 265 MB at peak). Before the limit the last file took 53 seconds. The limit is a choice: a log of more than 500000 joined gzip files has to be split. A decoder that is reset instead of set up again was tried; the reset of the decompressor window costs about as much (2.7 microseconds per member, 1000000 empty members in 2.8 seconds), so it was not kept.
- Messages. The `--help` text, the report and most messages are Dutch; these are English: `file is larger than 268435456 bytes`, `gzip: truncated gzip header`, `gzip: trailing data is not a gzip member`, `stream did not contain valid UTF-8`, `overflow`, the usage errors of the argument parser, and the operating-system text after a file name or `stdout:`. The README says so. Translating them is a product decision.
- A log with no data rows (an empty file, or only comment and `#Fields:` lines) gives `geen verkeer` and exit 0, also with `--strict`. Measured with `empty.log` (0 bytes) and a log with only a `#Fields:` line.
- With `--strict` and a stdout that cannot be written the exit code is 2 (the report could not be written), not 3.
- A `#` line with a tab inside the log is a data row, so `#Version: x<TAB>y` after the `#Fields:` line is an error (`2 velden, #Fields heeft 5`). Real CloudFront header lines have no tab.
- Binary. The release binary built on this host asks for GLIBC_2.34 (`objdump -T`). The musl release build succeeds and prints the quick start report byte for byte. No release binaries are published. No minimum version of macOS or Windows is known.
- "No network call" is checked only through the dependency list (`manifest_has_no_aws_sdk`) and a search of the source. No test watches the system calls.
- A usage error for a short option such as `-\xff\nFAKE` (a dash, then bytes that are not UTF-8) names only the character clap reads as the option: `unexpected argument '-\'`. It is safe (no raw line break) and less informative than the longer forms.
- CI. The workflow was compared with a sister project: `persist-credentials: false` on every checkout, `--no-fail-fast` on the test steps, 30 minutes on the test jobs (the sister project allows 45), and a `glibc` job that runs `scripts/check-glibc.sh`. The workflows were not started, so the new job was not seen to run on GitHub; the script was run by hand against the release binary built here (highest symbol 2.34, exit 0).
- A closed stdout is only recognised on Linux. On another unix system it cannot be told apart from a readable `/dev/null`.
- `BIN LOG | true` gives exit 2 with `Broken pipe` because the reader has already closed the pipe when the report is written. A reader that is slower to close could give exit 0.
- Test strength. No mutation run was made. The tests named above as pins describe behaviour that was already there. The claim that memory use is about 24 times the size of a text log is not pinned by any test, so a change of that number in the README would not be caught.
- Run time. The checks for a repeated field name, a repeated region and a label are linear. The time of a price file also grows with regions times months. Measured with the release binary and a log that has one row in every month from 1970 to 2100 (1572 months): 10000 regions 1.0 s, 40000 regions 13.2 s, 80000 regions 32.7 s. With a log of one month and 200000 different paths, a price file of 400000 regions took 2.5 s. A `#Fields:` line of 2000000 names (16.9 MB) took 0.4 s and 120 MB; a price file of 400000 regions and labels (22 MB) took 1.7 s and 222 MB with a one-line log. The built-in file has 8 regions. Making the cost per month independent of the number of regions is not done; a real price file has few regions, so this is a product decision.
- The tab separated report writes a path that starts with `=`, `+`, `-` or `@` unchanged (`=1+1` was written as `=1+1<TAB>5<TAB>1<TAB>0.000000...`). It is meant for a terminal; the README says so. Writing such a path differently is a product decision.
- Tests write their temporary files to the temporary directory with the process number in the name and remove them after the assertions, so a failing assertion can leave a file there. The file names do not collide between runs.
- The tests of the run time (`a_fields_line_with_many_names_is_read_in_linear_time` and `a_price_file_with_many_regions_and_labels_is_read_in_linear_time`) stop the binary after 20 seconds, a limit that a debug build on a very slow machine could approach: the fixed code needed under a second in debug builds here.
- Money rules. The total is the exact sum of the micro-USD amounts, rounded once per slice (see the README). The formulas and the rates were not changed in this record.

## Open points

- Whether the size limit should also apply to the unpacked size of a gzip file (a limit equal to the file limit would refuse a log of 30 MB gzip that unpacks to 300 MB).
- Whether the English messages should be translated, or the README sentence stays as it is.
- Whether an empty log should be a warning or, with `--strict`, an error.
- Whether the work per month should be made independent of the number of regions in the price file, and whether a path that looks like a formula should be written differently in the tab separated report.
- A test that runs the `cargo install --locked --path .` line.

## Claim to test

Every statement of `README.md` with the test that covers it. "Gap" means no test covers it, or only a part of it.

| # | Statement | Test | Status |
| --- | --- | --- | --- |
| R1 | Reads local files only, no AWS SDK, no network client | `manifest_has_no_aws_sdk` | gap: dependency list only, no network test |
| R2 | `cargo install --locked --path .` installs the binary | none | gap |
| R3 | The usage line and the options | `help_lists_log_files`, `help_long_options_match_the_man_page_and_unknown_option_is_not_a_panic` | tested |
| R4 | The built-in `data/prices.txt` and `data/edges.txt` apply without `--prices` and `--edges`; the files can be replaced | `embedded_source_and_date`, `embedded_edges_are_unique_and_priced`, `custom_prices_file_is_used_for_the_report`, `custom_edges_file_prices_every_edge_so_strict_exits_0`, `edges_file_with_unknown_price_region_exits_2` | tested |
| R5 | `--format` is `text` by default, and `json` is available | `json_format_on_the_fixture` | tested |
| R6 | `--month` keeps one month; an invalid month is exit 2 | `month_filter_keeps_march`, `bad_month_exits_2`, `requested_month_that_is_invalid_is_rejected` | tested |
| R7 | A missing log, an unreadable file or a file that is not a regular file is exit 2 | `missing_log_exits_2`, `unreadable_log_exits_2`, `prices_directory_exits_2`, `prices_fifo_exits_2_without_blocking`, `log_fifo_exits_2_without_blocking` | tested |
| R8 | `--strict` prints the report and exits 3 for unpriced bytes or requests | `strict_exits_3_and_still_prints_onbekend`, `custom_edges_file_prices_every_edge_so_strict_exits_0` | tested |
| R9 | A regular file above 268435456 bytes is refused before it is read (log, `--prices`, `--edges`); a file of exactly that size is read | `oversized_regular_file_exits_2_before_reading`, `a_file_of_exactly_the_size_limit_is_read_and_one_byte_more_is_refused` | tested |
| R10 | A pipe on stdin is read and counted, and above 268435456 bytes it stops with exit 2 | `log_through_pipe_to_dev_stdin_counts_like_the_file`, `oversized_pipe_to_dev_stdin_exits_2_without_hanging` | tested |
| R11 | On Linux a closed stdout is exit 2; `1<>/dev/null` and `>/dev/null` stay normal output | `stdout_closed_full_and_read_write_null_are_told_apart`, `closed_or_full_stdout_exits_2_without_panic`, `stdout_write_error_exits_2`, `help_and_version_to_closed_or_full_stdout_exit_2` | tested |
| R12 | On another unix system a closed stdout is not told from `/dev/null` | none | gap: not run |
| R13 | A log is gzip when named `.gz` (any case); text otherwise | `gzip_log_is_read`, `gzip_is_recognised_by_its_first_bytes_and_by_an_upper_case_name` | tested |
| R14 | A log is gzip when its first bytes are `1f 8b`, without a `.gz` name | `gzip_is_recognised_by_its_first_bytes_and_by_an_upper_case_name` | tested |
| R15 | Several gzip members, a BOM per member, zero padding ignored, other trailing data an error | `gzip_with_two_members_counts_both`, `gzip_second_member_with_bom_and_zero_padding`, `gzip_member_then_junk_without_padding`, `gzip_truncated_header_differs_from_truncated_body` | tested |
| R16 | Lines end in LF, CRLF or a lone CR | `crlf_comments_and_blank_lines`, `cr_only_lines_count_the_bytes` | tested |
| R17 | `#Fields:` sets the columns; five fields are required in any order; a repeated name is an error | `parses_tab_separated_standard_log`, `field_order_does_not_matter`, `missing_required_field_is_an_error`, `missing_field_error_text_names_each_required_field`, `repeated_field_name_is_an_error`, `repeated_field_name_exits_2` | tested |
| R18 | A wrong field count, an invalid date or a non-numeric `sc-bytes` is an error with file and line number | `bad_sc_bytes_names_the_line`, `log_error_texts_are_fixed`, `plus_date_exits_2` | tested |
| R19 | The rates and the check date are in `data/prices.txt`, source and date 2026-11-09 | `embedded_source_and_date`, `embedded_rates_match_published_table`, `readme_lists_source_url_and_check_date` | tested |
| R20 | The cost comes from `sc-bytes`, not `sc-content-len`; HTTP 404 counts; the path is `cs-uri-stem` | `cost_basis_is_sc_bytes_not_sc_content_len`, `http_status_404_still_counts`, `uri_stem_is_the_path` | tested |
| R21 | The region comes from the first three letters of `x-edge-location`; unknown stays `onbekend`; China is `onbekend` | `ams_pop_is_europe`, `iad_pop_is_us`, `unknown_iata_is_onbekend`, `malformed_location_is_onbekend`, `china_airports_are_onbekend` | tested |
| R22 | The report says the region is the edge, not the viewer's country | `text_report_says_edge_is_not_the_viewer` | tested |
| R23 | 1 GB is 1073741824 bytes; the first 1 TB is free for all regions together; 1 GB above it in the US costs 0.085 USD | `one_gb_above_the_free_terabyte_uses_the_us_rate`, `one_tebibyte_in_one_region_is_free`, `free_terabyte_comes_off_the_dearer_region` | tested |
| R24 | The first 10000000 requests per month are free together; HTTPS before HTTP at equal rate | `requests_inside_the_free_allowance_cost_nothing`, `https_above_the_free_requests_uses_the_table`, `equal_request_rates_waive_https_before_http` | tested |
| R25 | Every month has its own allowance | `months_each_get_their_own_free_terabyte` | tested |
| R26 | Unknown-edge bytes and requests, and other protocols, do not use up the allowance | `unknown_bytes_do_not_consume_the_free_terabyte`, `unknown_requests_do_not_consume_the_free_requests`, `other_protocol_does_not_consume_free_requests` | tested |
| R27 | The same path given twice counts twice | `same_log_twice_counts_twice` | tested |
| R28 | No flat-rate, no discount, no Savings Bundle (the report line) | `text_report_says_edge_is_not_the_viewer` (asserts the line) | tested |
| R29 | The amount is shared over the paths by bytes and by request count, remainders by the stated order, paths add up to the region amount | `remainder_micros_go_to_the_larger_remainder`, `a_larger_remainder_beats_a_larger_weight`, `equal_remainder_goes_to_the_larger_weight`, `equal_weight_remainder_goes_to_the_earlier_path`, `path_costs_sum_to_the_month_total` | tested |
| R30 | The split does not follow the order of the log lines | `the_order_of_the_log_lines_does_not_change_the_report` | tested |
| R31 | Text shows six decimals, a free amount is `0.000000`, an unpriced region shows `-`; JSON writes micro-USD | `json_format_on_the_fixture`, `text_report_marks_onbekend_unpriced` | tested |
| R32 | The quick start log and output are the files in `examples/` and what the binary prints | `quick_start_log_and_report_match_the_examples_and_the_binary` | tested |
| R33 | The test count in the README and this page | `quality_and_readme_share_suite_count` | tested (counts `#[test]` lines, does not run them) |
| R34 | The claims table names real tests | `readme_claims_name_real_tests` | tested |
| R35 | License MIT OR Apache-2.0 | `licenses_match_the_manifest` | tested |
| R36 | The help text and most messages are Dutch, a few are English | `help_lists_log_files` (help only), `log_error_texts_are_fixed`, `broken_gzip_error_text_is_fixed`, `overflowing_byte_count_names_the_log_and_exits_2`, `oversized_regular_file_exits_2_before_reading`, `invalid_utf8_log_exits_2` | tested in part: the usage errors of the parser are not pinned |
| R37 | The man page holds the `--help` text and the same long options | `help_long_options_match_the_man_page_and_unknown_option_is_not_a_panic` | tested |
| R38 | Dates and `--month` run from 1970 to 2100, and are real calendar days and months | `dates_and_months_are_accepted_from_1970_to_2100_only`, `leap_days_and_month_numbers_follow_the_calendar` | tested |
| R39 | `sc-bytes` is an unsigned 64-bit number; a sum that does not fit is `<log>: overflow` with exit 2 | `sc_bytes_is_an_unsigned_64_bit_number_and_a_sum_that_does_not_fit_is_an_error`, `overflowing_byte_count_names_the_log_and_exits_2` | tested |
| R40 | Every billable slice is rounded half up to one micro-USD; sums are not rounded again | `rounds_half_up`, `rounds_below_half_down`, `path_costs_sum_to_the_month_total` | tested in part: the rule of the function, not the choice of slices |
| R41 | An empty log or a header-only log is `geen verkeer`, exit 0, also with `--strict` | `an_empty_log_and_a_header_only_log_succeed_even_with_strict` | tested |
| R42 | A `#` line without a tab is a comment, with a tab a data row | `a_hash_line_that_holds_a_tab_is_a_data_row`, `crlf_comments_and_blank_lines` | tested |
| R42a | A data row may end in one tab; two are a field too many | `a_single_trailing_tab_on_a_data_row_is_tolerated` | tested |
| R43 | Memory use is about 24 times the size of a text log, and a gzip file is not limited after unpacking | none | gap: measured by hand (see Known limits), no test |
| R44 | The format of the price file | `price_file_error_texts_are_fixed`, `price_file_value_error_texts_are_fixed`, `custom_prices_file_is_used_for_the_report`, `embedded_rates_match_published_table` | tested |
| R45 | The format of the edge map | `edge_map_error_texts_are_fixed`, `custom_edges_file_prices_every_edge_so_strict_exits_0`, `edges_file_with_unknown_price_region_exits_2` | tested |
| R46 | Measured on Linux only; Windows and macOS not run | none | gap: stated as not run |
| R46a | The release binary needs GLIBC_2.34; the musl build has no glibc requirement and prints the quick start report; no macOS or Windows minimum is measured | none | gap: measured by hand (`objdump -T`, musl build, `cmp` against `examples/access.report.txt`), no test |
| R47 | A usage error does not print the argument's escape sequence, CR, bidi characters or line breaks as raw text; an argument that is not UTF-8 is shown as bytes (`\xff`) | `clap_error_with_an_escape_sequence_and_a_line_break_stays_on_one_line`, `clap_error_escapes_bidi_cr_and_newline_in_user_input`, `clap_error_with_a_non_utf8_argument_and_a_line_break_stays_on_one_line`, `non_utf8_month_value_keeps_clap_lines_and_non_utf8_option_shows_its_bytes` | tested |
| R48 | A gzip member may end inside a UTF-8 character | `gzip_members_may_split_a_utf8_character` | tested |
| R49 | The public text files, the manifest, the workflow, the sources, the tests and the examples hold no local path, wrapper option, working-note name or internal tool name | `public_docs_have_no_internal_names_or_local_paths`, `package_metadata_workflows_sources_and_tests_have_no_internal_names_either`, `the_check_itself_finds_each_forbidden_word`, `a_path_that_is_not_there_is_skipped` | tested |
| R50 | The man page and the README have the usage-error sentence | `help_long_options_match_the_man_page_and_unknown_option_is_not_a_panic` (checks the sentence in both) | tested |
| R51 | The tests also pass in the unpacked package | `a_path_that_is_not_there_is_skipped`, `package_metadata_workflows_sources_and_tests_have_no_internal_names_either` | tested in part: the run in the unpacked package was made by hand (see Commands), not by a test |
| R52 | The checks for a repeated field name, a repeated region and a label take time in proportion to the size of the file (a `#Fields:` line of 2000000 names 0.4 s; 400000 regions 1.7 s) | `a_fields_line_with_many_names_is_read_in_linear_time`, `a_price_file_with_many_regions_and_labels_is_read_in_linear_time` | tested in part: the tests use 200000 names and 100000 regions and a limit of 20 seconds; the larger figures were measured by hand |
| R53 | The time of a price file also grows with regions times months (40000 regions and 1572 months 13 s) | none | gap: measured by hand (see Known limits), no test |
| R54 | A plain-text file named `.GZ` is refused as a bad gzip file | `a_plain_file_with_an_upper_case_gz_name_is_refused_as_gzip` | tested |
| R55 | A path that starts with `=`, `+`, `-` or `@` is written unchanged in the report | none | gap: measured by hand (`=1+1`), no test |
| R57 | A gzip file holds at most 500000 members; a file under the size limit is never slower than about 2 seconds for that reason | `a_gzip_file_of_more_members_than_the_maximum_is_refused`, `many_empty_gzip_members_are_read_in_time`, `many_gzip_members_with_a_bom_each_are_read_in_time`, `many_gzip_members_with_every_header_part_are_read_in_time` | tested |
| R56 | The descriptions of `--help` are Dutch, its headings are English | `help_lists_log_files` (the Dutch description), `help_long_options_match_the_man_page_and_unknown_option_is_not_a_panic` (the English headings are part of the verbatim text) | tested |
| R58 | The report says it is an estimate: one `Estimate from list prices` line, `estimated_usd_month=` in text, `"estimate": true` and `estimated_usd_month_micros` in JSON | `text_labels_the_monthly_cost_as_an_estimate`, `json_is_marked_as_an_estimate_and_names_the_monthly_cost_alike`, `an_empty_report_is_still_an_estimate` | tested |
| R63 | Every JSON report starts with `"schema_version": 1` (also with `--output`, `--strict`, `--month`, an empty log); an error prints no JSON, only a stderr line and exit 2 | `every_json_report_has_schema_version_1_first`, `schema_version_is_also_there_with_strict_quiet_verbose_and_as_of`, `schema_version_is_also_in_a_json_file_from_output`, `an_error_with_json_selected_prints_no_json` | tested |
| R59 | Prices more than 180 days older than `--as-of` (default today) give one warning on stderr with their date and exit 0; exactly 180 days do not warn; `--as-of` must be a real date | `prices_older_than_180_days_warn_on_stderr_and_exit_0`, `prices_of_exactly_180_days_do_not_warn`, `the_built_in_prices_warn_when_as_of_is_far_ahead_and_not_when_close`, `as_of_must_be_a_real_date`, `price_age_is_over_180_days_only`, `day_numbers_follow_the_calendar` | tested |
| R60 | A log in the documented standard-log shape (33 fields, `-` values, percent-encoded paths, only the needed fields, header only, two logs together) gives the exact expected amounts | `the_full_documented_log_costs_exactly_one_gb_above_the_free_terabyte`, `the_full_documented_log_in_json_has_the_same_amount_in_micro_usd`, `a_log_with_only_the_needed_fields_is_read_and_two_logs_add_up`, `a_log_with_a_header_and_no_rows_is_an_empty_report` | tested |
| R61 | `--quiet` drops warnings but keeps the report, errors and the strict exit; `--verbose` adds progress on stderr only; both together are exit 2 | `quiet_keeps_the_report_and_drops_the_price_age_warning`, `quiet_still_reports_errors_and_the_strict_exit`, `verbose_adds_progress_on_stderr_and_leaves_stdout_alone`, `quiet_and_verbose_together_are_a_usage_error` | tested |
| R62 | `--md` prints the same amounts as text and JSON in tables, escapes `|` and `\` in cells, carries the estimate line, and conflicts with `--format` | `markdown_has_the_same_amounts_as_text_and_json`, `markdown_escapes_pipes_and_backslashes_in_cells`, `markdown_for_an_empty_log_and_the_conflict_with_format` | tested |
| R64 | SIGINT ends the run with exit 130 and the line `cdn-egress-bill: interrupted by SIGINT`; the report is written only once it is complete, but a signal during the write can leave part of it on stdout (the test blocks the write on a full pipe and checks exit and stderr, not the bytes on stdout) | `sigint_while_the_report_is_blocked_on_a_full_pipe_exits_130` | tested (a full pipe holds the report; a signal during the computation uses the same handler and was not timed) |
| R65 | `--output` writes the whole report through a temporary file and one rename, replaces a file or a symbolic link without following it, refuses a directory, and exits 2 with a message when it cannot write | `output_file_holds_exactly_the_report_that_stdout_would_have`, `an_existing_file_is_replaced_by_the_whole_new_report`, `a_missing_parent_directory_is_exit_2_and_is_not_created`, `a_directory_as_output_is_refused_and_kept`, `a_failing_run_leaves_an_existing_output_file_alone`, `a_symlink_as_output_is_replaced_and_its_target_is_not_written`, `a_name_with_control_characters_is_escaped_in_the_error`, `strict_still_exits_3_and_writes_the_file`, `output_without_a_value_is_a_usage_error` | tested; a SIGINT held back during the rename is not tested (gap) |
| R66 | `--version` is the crate version, with the short commit in a debug build from a git checkout | `version_is_the_crate_version_and_in_a_debug_checkout_also_the_commit` | tested |
| R67 | Each command of `examples/README.md` gives the stored text, Markdown and JSON output | `each_example_command_gives_its_expected_output`, `the_examples_readme_names_every_command_and_file` | tested |
| R68 | The release archive holds every file that its documents link to or name: `QUALITY.nl.md`, `SECURITY.md`, `data/prices.txt` and `data/edges.txt` | `every_relative_link_in_the_packaged_documents_points_into_the_archive` | tested on the `cp` lines; the archive itself was not built (not run) |
