# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2026-11-26

### Changed

- Version number raised from 1.0.0 to 1.0.1. The program behaves the same as 1.0.0; only the version number in the manifest, the documentation and the tests changed.

## [1.0.0] - 2026-11-15

Changes made after 2026-11-15; the version number did not change.

### Added

- `THIRD-PARTY-LICENSES.html`: the licenses of the third-party crates in the binary, generated with `cargo about generate --locked about.hbs`.
- QUALITY.md and QUALITY.nl.md state under the title which code commit the measurements apply to.

- Documentation standard: `README.nl.md`, `CONTRIBUTING.md`, `QUALITY.nl.md`, `examples/README.md` with stored text, Markdown and JSON output (tested), `about.hbs` and `about.toml` for the third-party license listing, `.github/workflows/release.yml`, a Support and a Contact section, and the README headings Installation, Quick start, Options and usage, Example, Exit codes.

- README sections `Prices and responsibility` and `Maintenance` (and `Prijzen en verantwoordelijkheid` and `Onderhoud` in the Dutch README), checked by a test in `tests/docs_clean.rs`. The suite has 228 tests.

- `build.rs`: a debug build from a git checkout of this crate prints the short commit after the version in `--version`. A release build or an unpacked package prints the version only.

- `"schema_version": 1` is the first key of every JSON report. An error prints no JSON.

- `--output FILE` writes the report to a file instead of stdout, through a temporary file in the same directory and one rename. A symbolic link is replaced, not followed; a directory, a missing parent or a write error is exit 2. A SIGINT during that write is held until the rename is done, then exit 130.

- SIGINT (Ctrl-C) ends the run with exit 130 and the stderr line `cdn-egress-bill: interrupted by SIGINT`; the report is written to stdout only once it is complete in memory, so a SIGINT during the write of a report larger than the pipe buffer can leave part of it on stdout. The handler only calls `write` and `_exit`; it is a second small `unsafe` module.

- `--md`: the report as Markdown tables with the same amounts as the text report and the estimate line. `|` and `\` in a cell are escaped. It cannot be combined with `--format` (exit 2).

- `--quiet` (only the report and errors, no price-age warning) and `--verbose` (progress on stderr). Both together are a usage error, exit 2.

- A test file with the input shape of the CloudFront standard log as the documentation describes it (33 fields, `-` for no value, percent-encoded paths, only the needed fields, a header without rows, two logs in one run), with exact expected amounts.

- `--as-of YYYY-MM-DD` and a warning on stderr when the `checked_on` date of the prices is more than 180 days before it (default: today, UTC). The exit code stays 0; a value that is not a real date is exit 2.

- The report says it is an estimate from list prices. Text prints one `Estimate from list prices, not your AWS invoice (free tier, credits, discounts not included)` line and labels the cost of each month `estimated_usd_month=` (it was `totaal_usd:`). JSON has `"estimate": true` at the top level, and the month key `total_usd_micros` is now `estimated_usd_month_micros`.

- On Linux, a stdout that was already closed when the program started is exit 2 with a message. `1<>/dev/null` and `>/dev/null` stay normal output. A small module records at start whether file descriptor 1 was closed, because Rust replaces a closed descriptor with `/dev/null` before `main`; it holds the only `unsafe` in the crate, one read-only `fcntl` call.
- A log, `--prices` file or `--edges` file above 268435456 bytes is refused with exit 2. A regular file is checked before it is read. A pipe on stdin has no length, so the same limit is applied while it is read.
- A log may be the program's own stdin pipe (for example `/dev/stdin`); any other file that is not a regular file is refused with exit 2 without blocking.
- A gzip log may hold several members. Each member is read, each member may start with a BOM, and zero bytes after a member are ignored. Other data after a member is an error, and a truncated gzip header has its own message.
- A log line may end in LF, CRLF or a lone CR.
- A `#Fields:` line with a no-break space is an error. A repeated field name is an error.
- A price file with a BOM, with a Unicode whitespace character other than space, tab or CR, or with two `label` lines for the same region is an error. An edge map with such a whitespace character is an error.
- Messages for an overflow while pricing now name the log files that took part, not their text.
- `homepage` and an `exclude` list in `Cargo.toml`. The package leaves out `.github/` and `.gitattributes`, and holds the man page and `SECURITY.md`.
- `man/cdn-egress-bill.1`, `SECURITY.md` and `tests/handbook_help.rs`, which checks that the man page holds the `--help` text verbatim and has the same long options.
- `examples/access.log` and `examples/access.report.txt`, the README quick start log and its report, checked against the binary by a test.
- `tests/docs_clean.rs`, which fails when a public text file, the manifest, the workflow, the sources, the tests, the examples or the data files hold a local path, a wrapper option, the name of a working-note file or the name of an internal tool.
- The README says what the release binary needs (glibc 2.34, or the musl build) and that a data row may end in one tab, and it says how a usage error shows what was typed. The man page has the same sentence about usage errors.
- The README says that, after a single dash, a name that is not UTF-8 is reported by one character only. The man page and the README sentences about usage errors are checked by `tests/handbook_help.rs`.
- Tests for the calendar rules (the century rule, month 00 and 13), the single trailing tab, a file of exactly 268435456 bytes, a larger remainder against a larger weight, a usage error with an argument that is not UTF-8, and a path that is not there in the docs check.
- The README and `SECURITY.md` state the run time of the checks and its measured limits (a `#Fields:` line, a price file, and the product of regions and months), and the README says that a path that starts with `=`, `+`, `-` or `@` is written unchanged in the tab separated report.
- A CI job `glibc` (ubuntu 22.04) runs `scripts/check-glibc.sh`, which fails when the release binary needs a glibc symbol newer than 2.34, and a `dependabot.yml`. The package leaves out `scripts/`.
- Tests for the gzip member limit and for many empty members, with a BOM or a name in each member, and for a damaged member after many empty ones. The dependencies are built with optimisation in the dev profile, so the tests of many members are not slow in a debug build.
- Tests: a `#Fields:` line of 200000 names and a price file of 100000 regions with labels are read within 20 seconds (a repeated name or region at the end is still found); a plain-text file named `.GZ` is refused as a bad gzip file; the man page and the README sentences on usage errors are checked on the clause about an argument that is not UTF-8.
- The README lists the input rules and measured limits: the date range, the `sc-bytes` range, the rounding rule, the empty log, the `#` line with a tab, memory use and the gzip size, and the format of the price file and the edge map. It also says that some messages are English.

### Changed

- A gzip log of more than 500000 members is refused with exit 2 (`gzip: more than 500000 members`). Every member costs about 4 microseconds, so a file of 13421700 empty members (268434080 bytes) took 53 seconds; now it is refused after 2.0 seconds, and a file of 499999 empty members is read in 1.8 seconds.

- `--help` and `--version` go through the same checked write as the report. A closed or full stdout is exit 2, where it was exit 0.
- A usage error from the command line is printed with the user's own input escaped (as `\u000a` or `\u202e`), so a bidi character, CR or newline in an argument does not reach the terminal.
- Error messages escape control characters and invisible or bidi characters. File names in errors are escaped too; a name that is not UTF-8 is shown as byte escapes.
- The text report escapes the price source URL, the check date and the region labels. Text and JSON escape the invisible and bidi characters listed in the code, also outside the Basic Multilingual Plane: text writes `\u{..}`, JSON writes UTF-16 `\uXXXX` units.
- A line that starts with `#` but holds a tab is a data row, not a comment.
- The `#Fields:` line is split on space, tab, CR and LF only.
- The library denies `unsafe` code instead of forbidding it, with one allowed module (see Added). The binary still forbids it.
- The test count marker in the README and in QUALITY.md is an HTML comment, so it does not show in the rendered pages.
- Two tests clean up their temporary files before they assert, so a failing assertion leaves nothing behind.
- Fixed texts of the log, price file, edge map, amount and internal errors are now covered by tests.
- The CI workflow sets `persist-credentials: false` on every checkout and runs the test steps with `--no-fail-fast`. `deny.toml` has a two-line comment on top.
- `tests/docs_clean.rs` also looks for the words of five more internal names, reads `Cargo.lock`, the licence files and `scripts/`, and fails when `.github/`, `src/`, `tests/`, `examples/` or `data/` exists but its main file was not read. `.gitattributes` keeps `tests/fixtures/` byte for byte.
- The README and the man page no longer say that the whole `--help` text is Dutch: the descriptions are, the headings and fixed words (`Usage:`, `Options:`, `Print help`) are English. The man page says, as the README does, that for an argument that is not UTF-8 the message of the argument parser is printed as it is.
- The package no longer lists the names of working-note files in `exclude`; `.gitattributes` now sets `* text=auto eol=lf`, so the man page, the examples and the golden files keep their bytes in a checkout with CRLF. CI gives the audit and licenses jobs a 15 minute limit and the test and msrv jobs 30 minutes, and `deny.toml` allows unused allowed licenses.
- The tests no longer depend on the system clock: every test that runs the binary and needs an empty stderr gives a fixed `--as-of`. The suite still has 228 tests; they also pass with the clock set 400 days ahead.

### Fixed

- The release archives (the downloads, not the crates.io package) now hold `QUALITY.nl.md`, `SECURITY.md` and `data/`, which `README.nl.md` and both READMEs link to or name. The smoke step of the release workflow checks every relative link of the documents in the unpacked archive, and `tests/archive_links.rs` checks the same on the `cp` lines of the workflow. The suite has 229 tests. The workflow itself was not run.

- A `#Fields:` line with many names, and a price file with many `region` or `label` lines, no longer take time in proportion to the square of their size. A line of 50000 names (338953 bytes) took 2.8 seconds and one of 100000 names (688953 bytes) 13.4 seconds; both are now under 0.02 seconds, and a line of 2000000 names takes 0.4 seconds. A price file with 40000 extra regions took 2.2 seconds and now 0.07 seconds. The error texts and line numbers for a repeated name, region or label are unchanged.
- Pricing a month no longer goes over all paths again for every region of the price file. A log of 200000 paths with a price file of 10000 regions took 118 seconds and now 0.5 seconds. The report is byte for byte the same: the old and the new binary gave the same text and JSON for four logs, also with a price file of 10000 regions.
- A usage error whose argument holds an escape sequence followed by a line break, a CR or a bidi character is printed on one line with `\u....` for each of those characters. It used to print the rest of the argument as a line of its own. The same holds for an unknown option.
- A usage error whose argument is not UTF-8 and holds a line break is printed on one line too. The bytes of such an argument are shown as `\xff` and `\n`; before, the line break started a line of its own.
- A usage error for an argument that is not UTF-8 and has no other fault (for example `--month` with such a value) is printed in clap's own lines, as clap writes it. It was one line with `\u000a` in place of every line break of clap.
- A usage error ends after clap's own text, without an extra blank line.
- A gzip log in which one member ends in the middle of a UTF-8 character is read. The members are joined as bytes before the text is decoded; it was refused with `stream did not contain valid UTF-8`.
- `cargo test` in the unpacked package passes. The second check of `tests/docs_clean.rs` no longer fails on `.github/` and `.gitattributes`, which the package leaves out; a path that is not there is skipped.
- Tests that need a name that is not UTF-8 or an escape byte in a name run on Linux only, and the test that writes to a pipe on `/dev/stdin` no longer returns before the child is waited for, which clippy 1.85 refused.


### Changed

- Version 1.0.0. CI also runs `cargo audit` and `cargo deny --locked check licenses`.
- `--prices` and `--edges` refuse what is not a regular file and are read no further than the length their metadata reported.

## [0.1.0] - 2026-11-09

### Added

- First release. Estimates CloudFront egress cost per path and per price region from local standard logs, offline, without an AWS SDK.
