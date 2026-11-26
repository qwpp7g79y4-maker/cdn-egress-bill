# cdn-egress-bill

[![crates.io](https://img.shields.io/crates/v/cdn-egress-bill.svg)](https://crates.io/crates/cdn-egress-bill) [![downloads](https://img.shields.io/crates/d/cdn-egress-bill.svg)](https://crates.io/crates/cdn-egress-bill) [![CI](https://github.com/qwpp7g79y4-maker/cdn-egress-bill/actions/workflows/ci.yml/badge.svg)](https://github.com/qwpp7g79y4-maker/cdn-egress-bill/actions/workflows/ci.yml) [![PEPAI checked](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fqwpp7g79y4-maker%2Fcdn-egress-bill%2Fmain%2F.github%2Fbadge.json)](QUALITY.md) [![MSRV 1.85](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://github.com/qwpp7g79y4-maker/cdn-egress-bill/blob/main/Cargo.toml) [![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/qwpp7g79y4-maker/cdn-egress-bill#license)

Estimate CloudFront egress cost from standard log files. Reads local files only: there is no AWS SDK and no network client.

The tests and the measured numbers are in `QUALITY.md`.

## Why

A CloudFront standard log already lists the bytes sent for every request, the edge location and the path. cdn-egress-bill prices those logs on your machine, per price region and per path, with the free terabyte and the free requests applied. It does not call AWS and it does not need an account.

The descriptions in the `--help` text and the report are in Dutch, and are quoted in this file exactly as the program prints them. The headings and fixed words of `--help` (`Usage:`, `Options:`, `Print help`, `[default: text]`) come from the argument parser and are English. Most error messages are Dutch. A few are English: `file is larger than 268435456 bytes`, `gzip: truncated gzip header`, `gzip: trailing data is not a gzip member`, `stream did not contain valid UTF-8` and `overflow`. A usage error comes from the argument parser in English, and the text after a file name or after `stdout:` comes from the operating system.

## Installation

Version 1.0.0. Measured on Linux (x86_64) only; see [QUALITY.md](QUALITY.md) for what was and was not run. From crates.io, once 1.0.0 is published there:

```sh
cargo install cdn-egress-bill
```

Or build from source.

With Rust 1.85 or newer, from a clone of this repository:

```sh
cargo install --locked --path .
cdn-egress-bill --help
```

This crate is not published and has no git tag. Until then, install from this checkout. The tests that `cargo test --locked` runs are counted in [QUALITY.md](QUALITY.md).

<!-- SUITE_TESTS: 229 -->

## Quick start

A small log, with the columns `date`, `x-edge-location`, `sc-bytes`, `cs-uri-stem` and `cs-protocol`, and the report for it (the options are under Options and usage):

```sh
printf '#Version: 1.0\n#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n2026-03-01\tAMS50-C1\t1000\t/video.mp4\thttps\n2026-03-01\tIAD89-P1\t2000\t/img.png\thttp\n2026-03-02\tZZZ1-C1\t50\t/secret\thttps\n' > access.log
cdn-egress-bill access.log
```

The same log and report are in `examples/access.log` and `examples/access.report.txt`. The report is tab separated. The 50 bytes of `/secret` come from an edge that is not in the map, so that region is `onbekend` and is not priced (`-`):

```
bron: https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/
gecontroleerd: 2026-11-09
bytes_per_gb: 1073741824
gratis_data_bytes: 1099511627776
gratis_verzoeken: 10000000
Estimate from list prices, not your AWS invoice (free tier, credits, discounts not included)
geen flat-rate, geen korting, geen Savings Bundle
regio = prijsregio van de edge-locatie (x-edge-location), niet het land van de kijker

maand: 2026-03
per regio
regio	edges	bytes	http	https	overig	usd	berekend
US/Mexico/Canada	IAD	2000	1	0	0	0.000000	ja
Europe/Israel/Türkiye	AMS	1000	0	1	0	0.000000	ja
onbekend	ZZZ	50	0	1	0	-	nee
per pad
pad	bytes	verzoeken	usd	onberekend_bytes	onberekend_verzoeken
/img.png	2000	1	0.000000	0	0
/secret	50	1	0.000000	50	1
/video.mp4	1000	1	0.000000	0	0
estimated_usd_month=0.000000
gratis_data_toegepast: 3000
gratis_verzoeken_toegepast: 2
onberekend_bytes: 50
onberekend_verzoeken: 1
```

## Quickstart in 5 minutes

1. Build or install the tool (see Installation): `cargo install --locked --path .`.
2. Put a CloudFront standard log (text or gzip) in a folder, for example `access.log`.
3. Run `cdn-egress-bill access.log`: the output is an estimate from list prices, not an AWS invoice.
4. Prefer to count per month? Add `--month 2026-03`. Want a file? Add `--output report.txt`. Want Markdown or JSON? Use `--md` or `--format json`.
5. An error gives exit 2 and a message on stderr; see Exit codes.

## Options and usage

```
cdn-egress-bill [--prices FILE] [--edges FILE] [--format text|json] [--month YYYY-MM] [--as-of YYYY-MM-DD] [--md] [--output FILE] [--quiet | --verbose] [--strict] LOG [LOG ...]
```

`LOG` is a CloudFront standard log, as text or gzip. Without `--prices` and `--edges` the built-in files `data/prices.txt` and `data/edges.txt` apply. `--format` is `text` by default. `--month` keeps one month (UTC, `YYYY-MM`). `--as-of YYYY-MM-DD` (default: today, UTC) is the date the age of the prices is measured against: when the `checked_on` date of the price file (built-in or `--prices`) is more than 180 days before it, one warning with that date goes to stderr and the exit code stays 0. `--output FILE` writes the report to `FILE` instead of stdout: it is written to a temporary file in the same directory and renamed over `FILE`, so the file is the old one or the whole new one, never half of a report. An existing file is replaced, a symbolic link at `FILE` is replaced and not followed, and a directory is refused. A missing parent directory (it is not created), a directory or a write error is exit 2 with a message. `--strict` (exit 3) still writes the file. `--quiet` prints only the report and errors (no warning); `--verbose` adds progress lines on stderr (price date, each log, record and month counts) and leaves stdout as it is. Both together are a usage error, exit 2 (the usage-error code of this tool). `--version` prints the crate version; a debug build made in a git checkout of this crate adds the short commit, for example `cdn-egress-bill 1.0.0 (a1b2c3d)`, and a release build or an unpacked package prints the version only.

## Example

The report is an estimate from list prices, not your AWS invoice: the line `Estimate from list prices, ...` is printed once, each month ends its money with `estimated_usd_month=`, and the JSON starts with `"schema_version": 1` (the version of the JSON layout; it changes when a key is renamed, removed or changes meaning) and has `"estimate": true` at the top and `estimated_usd_month_micros` per month. `--md` prints the same report as Markdown (it cannot be combined with `--format`, exit 2): the same amounts as the text report in tables, with `|` and `\` in a cell written as `\|` and `\\`, and control characters as `\u000a`. The same log gives:

```
# CDN egress estimate

Estimate from list prices, not your AWS invoice (free tier, credits, discounts not included)

- bron: https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/
- gecontroleerd: 2026-11-09
- bytes_per_gb: 1073741824
- gratis_data_bytes: 1099511627776
- gratis_verzoeken: 10000000

geen flat-rate, geen korting, geen Savings Bundle

regio = prijsregio van de edge-locatie (x-edge-location), niet het land van de kijker

## 2026-03

### per regio

| regio | edges | bytes | http | https | overig | usd | berekend |
| --- | --- | --- | --- | --- | --- | --- | --- |
| US/Mexico/Canada | IAD | 2000 | 1 | 0 | 0 | 0.000000 | ja |
| Europe/Israel/Türkiye | AMS | 1000 | 0 | 1 | 0 | 0.000000 | ja |
| onbekend | ZZZ | 50 | 0 | 1 | 0 | - | nee |

### per pad

| pad | bytes | verzoeken | usd | onberekend_bytes | onberekend_verzoeken |
| --- | --- | --- | --- | --- | --- |
| /img.png | 2000 | 1 | 0.000000 | 0 | 0 |
| /secret | 50 | 1 | 0.000000 | 50 | 1 |
| /video.mp4 | 1000 | 1 | 0.000000 | 0 | 0 |

estimated_usd_month=0.000000

- gratis_data_toegepast: 3000
- gratis_verzoeken_toegepast: 2
- onberekend_bytes: 50
- onberekend_verzoeken: 1
```

Text shows USD with six decimals. A free amount is `0.000000`. An unpriced region shows `-`. JSON writes micro-USD as an integer (1 USD = 1000000).

## How it counts

A log is gzip when its file name ends in `.gz` (any case) or its first two bytes are `1f 8b`; otherwise it is UTF-8 text. A gzip file may hold several members: every member is read, every member may start with a BOM, and zero bytes after a member are ignored. Other data after a member is an error. A file of more than 500000 members is an error too (`gzip: more than 500000 members`, exit 2). Lines end in LF, CRLF or a lone CR. The `#Fields:` line sets the columns. `date`, `x-edge-location`, `sc-bytes`, `cs-uri-stem` and `cs-protocol` are required; the order does not matter, and a repeated field name is an error. A line with a different number of fields, an invalid date, or an `sc-bytes` that is not a number is an error with file and line number.

The rates and the check date are in `data/prices.txt`. That is the only price table. Source: https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/ . Checked on 2026-11-09.

The cost comes from `sc-bytes`. Every line counts, also HTTP 404. The path is `cs-uri-stem`, unchanged. `sc-content-len` does not count.

The price region comes from `x-edge-location`. The first three letters are the IATA code of the edge. The region is the price region of the edge location (`x-edge-location`), not the viewer's country. An unknown code is `onbekend` and is not guessed. China has a separate price list, is not in the map and stays `onbekend`.

One gigabyte is 1073741824 bytes. The first 1 TB per month is free for all regions together. The first 10000000 HTTP and HTTPS requests per month are free together. Every month has its own free allowance. Bytes and requests on an unknown edge do not use up that allowance. A protocol other than HTTP or HTTPS does not use up the free requests. When the request rate is equal, the free allowance comes off HTTPS first, before HTTP.

1 GB above that free terabyte in US/Mexico/Canada costs 0.085 USD. Further tiers are only in `data/prices.txt`.

The same path given twice as an argument counts twice.

No flat-rate, no discount, no Savings Bundle.

After the region costs, the amount is shared over the paths in proportion: data by bytes, requests by count. The split does not follow the order of the log lines. Remainder micros go to the largest remainder fraction, then the largest weight, then the path name, so that the paths add up to the region amount.

### Claims

| Statement | Test |
| --- | --- |
| 1 GB above the free terabyte in the US costs 0.085 USD | `one_gb_above_the_free_terabyte_uses_the_us_rate` |
| A log in the documented standard-log shape gives the exact amount | `the_full_documented_log_costs_exactly_one_gb_above_the_free_terabyte` |
| The estimate line, `estimated_usd_month=` and `"estimate": true` | `text_labels_the_monthly_cost_as_an_estimate` |
| Every JSON report starts with `"schema_version": 1`; an error prints no JSON | `every_json_report_has_schema_version_1_first`, `schema_version_is_also_there_with_strict_quiet_verbose_and_as_of`, `schema_version_is_also_in_a_json_file_from_output`, `an_error_with_json_selected_prints_no_json` |
| Prices older than 180 days warn on stderr, exit 0 | `prices_older_than_180_days_warn_on_stderr_and_exit_0` |
| SIGINT while the report is being written exits 130 with the interrupted line | `sigint_while_the_report_is_blocked_on_a_full_pipe_exits_130` |
| `--output` writes exactly the report stdout would have, by temporary file and rename | `output_file_holds_exactly_the_report_that_stdout_would_have`, `an_existing_file_is_replaced_by_the_whole_new_report`, `strict_still_exits_3_and_writes_the_file` |
| `--output` failures (missing parent, directory) are exit 2 and keep the old file | `a_missing_parent_directory_is_exit_2_and_is_not_created`, `a_directory_as_output_is_refused_and_kept`, `a_failing_run_leaves_an_existing_output_file_alone`, `a_name_with_control_characters_is_escaped_in_the_error` |
| `--output` does not follow a symbolic link | `a_symlink_as_output_is_replaced_and_its_target_is_not_written` |
| `--version` is the crate version, plus the commit in a debug checkout | `version_is_the_crate_version_and_in_a_debug_checkout_also_the_commit` |
| Each command in `examples/README.md` gives the stored text, Markdown and JSON output | `each_example_command_gives_its_expected_output`, `the_examples_readme_names_every_command_and_file` |
| 1 TB in one region is free | `one_tebibyte_in_one_region_is_free` |
| The free terabyte comes off the dearer region | `free_terabyte_comes_off_the_dearer_region` |
| HTTPS above 10000000 requests follows the table | `https_above_the_free_requests_uses_the_table` |
| The fixture has two regions and two paths | `fixture_has_two_regions_and_two_paths` |
| An unknown edge is called onbekend | `fixture_labels_unknown_edge_onbekend` |
| China airports are onbekend | `china_airports_are_onbekend` |
| The report says the edge is not the viewer's country | `text_report_says_edge_is_not_the_viewer` |
| Onbekend is not priced | `text_report_marks_onbekend_unpriced` |
| Remainder micros go to the larger remainder | `remainder_micros_go_to_the_larger_remainder` |
| Every month has its own free terabyte | `months_each_get_their_own_free_terabyte` |
| Strict exits 3 and still prints the report | `strict_exits_3_and_still_prints_onbekend` |
| The paths add up to the month total | `path_costs_sum_to_the_month_total` |
| On an equal remainder the micro goes to the larger weight | `equal_remainder_goes_to_the_larger_weight` |
| On an equal weight the micro goes to the earlier path name | `equal_weight_remainder_goes_to_the_earlier_path` |
| On an equal request rate the free allowance comes off HTTPS first | `equal_request_rates_waive_https_before_http` |
| The manifest has no AWS SDK | `manifest_has_no_aws_sdk` |
| The quick start log and report are the files in `examples/` and what the binary prints | `quick_start_log_and_report_match_the_examples_and_the_binary` |
| A gzip log is recognised by its first bytes or by a `.gz` name in any case | `gzip_is_recognised_by_its_first_bytes_and_by_an_upper_case_name` |
| A gzip member may end inside a UTF-8 character | `gzip_members_may_split_a_utf8_character` |
| The order of the log lines does not change the report | `the_order_of_the_log_lines_does_not_change_the_report` |
| Dates and `--month` run from 1970 to 2100 | `dates_and_months_are_accepted_from_1970_to_2100_only` |
| `sc-bytes` is an unsigned 64-bit number; a sum that does not fit is an error | `sc_bytes_is_an_unsigned_64_bit_number_and_a_sum_that_does_not_fit_is_an_error` |
| An empty log or a log with only a header is a report with no traffic, exit 0 | `an_empty_log_and_a_header_only_log_succeed_even_with_strict` |
| A `#` line that holds a tab is a data row | `a_hash_line_that_holds_a_tab_is_a_data_row` |
| A usage error with an escape sequence and a line break stays on its own lines | `clap_error_with_an_escape_sequence_and_a_line_break_stays_on_one_line` |
| A usage error with an argument that is not UTF-8 and holds a line break stays on its own lines | `clap_error_with_a_non_utf8_argument_and_a_line_break_stays_on_one_line` |
| A usage error for an argument that is not UTF-8 keeps clap's own lines, and shows the bytes as `\xff` | `non_utf8_month_value_keeps_clap_lines_and_non_utf8_option_shows_its_bytes` |
| A larger remainder beats a larger weight | `a_larger_remainder_beats_a_larger_weight` |
| Leap days and month numbers follow the calendar | `leap_days_and_month_numbers_follow_the_calendar` |
| One empty field after the last tab of a data row is ignored, two are an error | `a_single_trailing_tab_on_a_data_row_is_tolerated` |
| A file of exactly 268435456 bytes is read, one byte more is refused | `a_file_of_exactly_the_size_limit_is_read_and_one_byte_more_is_refused` |
| A plain-text file named `.GZ` is refused as a bad gzip file, so the name rule is applied on its own | `a_plain_file_with_an_upper_case_gz_name_is_refused_as_gzip` |
| A gzip file of 500000 members is read, one of 500001 members is refused with exit 2, and 200000 empty members, with or without a BOM or a name in each, are read within 30 seconds | `a_gzip_file_of_more_members_than_the_maximum_is_refused`, `many_empty_gzip_members_are_read_in_time`, `many_gzip_members_with_a_bom_each_are_read_in_time`, `many_gzip_members_with_every_header_part_are_read_in_time` |
| Members with a name, a comment or an extra field give the report of one member, and a damaged member after many empty ones gives the same message as alone | `gzip_members_with_header_parts_give_the_report_of_one_member`, `a_damaged_member_after_empty_members_is_refused_with_the_same_text` |
| A `#Fields:` line of 200000 names, and a price file of 100000 regions with labels, are read within the time limit of the test, and a repeated name or region at the end is still found | `a_fields_line_with_many_names_is_read_in_linear_time`, `a_price_file_with_many_regions_and_labels_is_read_in_linear_time` |

## Input rules and limits

Measured with the release binary on Linux; the commands and results are in [QUALITY.md](QUALITY.md).

- Dates: `date` is `YYYY-MM-DD`, a real calendar day from 1970-01-01 to 2100-12-31. `--month` takes the same years, 1970-01 to 2100-12. Anything else is exit 2 with the file and line number.
- `sc-bytes` is decimal digits only, 0 to 18446744073709551615. A sign, a fraction, a hexadecimal number, an empty field or a number above that is exit 2. A sum that does not fit the amount arithmetic stops the run with `<log>: overflow`, exit 2, and no report; 300 lines of 18446744073709551615 bytes in one region do that.
- Rounding: every billable slice is rounded half up to one micro-USD (1/1000000 USD). A slice is one price step of one region for data, and one protocol of one region for requests. Region amounts and the month total are sums of those micro-USD amounts and are not rounded again.
- A log with no data rows (empty, or only comment and `#Fields:` lines) is not an error: the report says `geen verkeer` and the exit code is 0, also with `--strict`.
- A data row may end in one tab: one empty field after the last tab is ignored, two are a field too many.
- A line that starts with `#` and has no tab is a comment. A line that starts with `#` and holds a tab is a data row, because a field such as `cs(Referer)` can start with `#`. A `#Version: x` line with a tab in it in the middle of a log is therefore an error with the line number.
- Run time of a gzip log with very many members. Every member sets up a decoder, which costs about 4 microseconds, also for an empty member or a member that holds only a BOM. Measured with the release binary: a file of 124999, 249999 and 499999 empty members after a header member (2.5 MB, 5.0 MB, 10.0 MB) took 0.44, 0.89 and 1.84 seconds, and with a BOM in each member 0.46, 0.93 and 1.82 seconds, with at most 14 MB at peak. More than 500000 members is refused, so a file under the size limit takes about 2 seconds for this reason (a file of 13421700 members, 268434080 bytes, was refused after 2.0 seconds; it took 53 seconds before the limit). A log made by joining more than 500000 gzip files has to be split.
- Run time. The checks for a repeated field name, a repeated region and a label for a region take time in proportion to the size of the file. Measured with the release binary: a `#Fields:` line of 2000000 names (16.9 MB) took 0.4 seconds, a price file of 400000 regions with labels (22 MB) took 1.7 seconds with a one-line log. The time of a price file also grows with the number of regions times the number of months in the log: 40000 regions and a log with 1572 months (every month from 1970 to 2100) took 13 seconds, 80000 regions took 33 seconds. The built-in price file has 8 regions.
- A log is read completely into memory. A text log took about 24 times its size at peak (a log of 50000093 bytes with 1345030 different paths: 1.2 GB, 3.6 seconds). The limit of 268435456 bytes is on the file as stored. It does not limit what a gzip file unpacks to: a 4.7 MB gzip of 1 GiB of zero bytes was unpacked completely, 2.1 GB at peak, before it was refused. Split very large logs, and do not read gzip files you do not trust on a small machine.
- The price file (`--prices`) holds lines of the form `source_url https://...`, `checked_on YYYY-MM-DD`, `bytes_per_gb N`, `free_data_bytes N`, `free_requests N`, `label ID TEXT` and `region ID R1 R2 R3 R4 R5 R6 R7 HTTP HTTPS`. `R1` to `R7` are the USD per GB of the seven data steps, `HTTP` and `HTTPS` are the USD per 10000 requests, with at most 6 decimals. Every key must be present once, `ID` is lower-case letters, digits and `_`, a `label` is optional and only once per region. Lines that start with `#` and empty lines are ignored. Fields are separated by spaces or tabs, and the file must not start with a BOM. `data/prices.txt` is the full example.
- The edge map (`--edges`) holds one `CODE REGION-ID` per line, with a three-letter IATA code, a code once only, and a region id that exists in the price file. `data/edges.txt` is the full example.
- Usage errors. What you typed is shown back with every control character, escape, line break, bidi and invisible character written as `\u000a`, `\u001b` and so on, so an argument cannot add a line to the message. An argument that is not UTF-8 is shown as bytes, for example `\xff`; when the only problem is that an argument is not UTF-8 (for example `--month` with such a value), clap's own message is printed, because it does not repeat the argument. After a single dash, such an argument can be reported by one character only (`unexpected argument '-\'`).
- System requirements. The release binary built on Linux with glibc needs GLIBC_2.34 or newer. A build for `x86_64-unknown-linux-musl` has no glibc requirement and printed the quick start report byte for byte. No minimum version of macOS or Windows is measured: the code was type-checked and linted for both, and no test ran there.
- On a platform other than Linux the tests that need `/dev/full`, `/proc`, a closed file descriptor or a name that is not UTF-8 do not run, and a closed stdout is not recognised (see above). Windows and macOS were not run.

## Exit codes

Exit code 0 is success. Exit code 130 is SIGINT (Ctrl-C): the tool prints `cdn-egress-bill: interrupted by SIGINT` on stderr, starts writing to stdout only when the whole report is computed (a SIGINT during the computation leaves stdout empty, but one that arrives while a report larger than the pipe buffer is being written can leave part of it on stdout), and never panics. With `--output`, a SIGINT during the final write waits for the rename, so the file is whole, and then exits 130. Exit code 2 is an input or usage error: an invalid month, a missing log, an unreadable file, or something that is not a regular file (except the own stdin pipe). `--strict` prints the report and exits 3 when there are unpriced bytes or requests. A regular file (log, `--prices` or `--edges`) above 268435456 bytes is refused with exit 2 before it is read. A pipe on stdin has no length: the same limit applies by counting what arrives, and above 268435456 bytes the read stops with the same error and exit 2. On Linux, a stdout that was closed at start is exit 2, while `1<>/dev/null` and `>/dev/null` stay normal output. On another unix system a closed stdout cannot be told apart from a readable `/dev/null`, and does not count as closed.

| Code | Meaning |
| --- | --- |
| 0 | Success |
| 2 | Input or usage error, including `--quiet` with `--verbose`, `--md` with `--format`, and an `--output` that cannot be written |
| 3 | `--strict` and some bytes or requests could not be priced (the report is still printed) |
| 130 | SIGINT (Ctrl-C) |

## Limitations

Version 1.0.0 prices CloudFront pay-as-you-go egress from standard logs only. There is no AWS SDK and no network call. The built-in rates are the ones in `data/prices.txt`, checked on 2026-11-09. China is not priced. There is no flat-rate, no discount and no Savings Bundle. An unknown edge is not guessed and its bytes and requests stay unpriced. The region is the edge, not the viewer. The same log given twice counts twice. A file above 268435456 bytes is refused, but a gzip file is not limited after unpacking. The help text and most messages of the program are Dutch, a few messages are English. The report is tab separated text meant for a terminal. A path that starts with `=`, `+`, `-` or `@` is written unchanged, so a spreadsheet that opens the output may read it as a formula; real CloudFront paths start with `/`. The numbers are an estimate, not a bill.

## Prices and responsibility

The built-in prices are a snapshot of the AWS CloudFront pay-as-you-go list prices, from https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/ , checked on 2026-11-09 (the `checked_on` line of `data/prices.txt`). Check the current prices yourself. You are responsible for any decision you take on the basis of the output. With `--prices FILE` you give your own price file.

## Maintenance

Report bugs through GitHub Issues of this repository (https://github.com/qwpp7g79y4-maker/cdn-egress-bill/issues). After a release the tool is dormant: a new version follows only for a reported bug, a request or a security problem, and security always comes first.

## Support

Bugs and questions: open an issue on GitHub (https://github.com/qwpp7g79y4-maker/cdn-egress-bill/issues). Please attach a small synthetic log, never a real CloudFront log. Security problems go through SECURITY.md.

## License

MIT OR Apache-2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.

The licenses of the third-party crates built into the binary are listed in [THIRD-PARTY-LICENSES.html](THIRD-PARTY-LICENSES.html).

## Contact

PEPAI, pep_ai@icloud.com, https://www.pepai.nl, X https://x.com/pepai_nl, Ko-fi https://ko-fi.com/pepai
