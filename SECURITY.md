# Security policy

## Supported versions

| Version | Supported |
| --- | --- |
| 1.0.x | Yes |

Security fixes go into the latest release only.

## Reporting a vulnerability

Please report security problems privately, not in a public issue: open the **Security** tab of this repository and choose **Report a vulnerability**. If that is not possible, write to pep_ai@icloud.com. Examples are a log line, path, edge code, price label, file name or command-line argument that can add a fake line or a terminal escape to the report or to an error, a way to make cdn-egress-bill write, run, or connect to something, a file or pipe that makes it read more than the stated size limit of a log, `--prices` or `--edges` file (the limit is on the file as stored, see the README for gzip), or hang, and input that changes an amount without an error.

Please include:

- the cdn-egress-bill version (the `version` field in `Cargo.toml`, or the commit you built) and your operating system;
- the exact command;
- a minimal synthetic log (text or gzip) that reproduces the problem, never a real CloudFront log;
- what cdn-egress-bill printed and what you expected.

We aim to respond within 7 days (best effort). Once a fix is released, the report can be credited in the changelog if you wish.

## Known limit

A gzip log is unpacked completely in memory; the size limit counts the file as stored. A report that only shows that a small gzip file unpacks to a lot of memory is already known and described in the README. A gzip file may hold at most 500000 members (each one sets up a decoder, about 4 microseconds; 1.8 seconds for 499999 empty members, measured); a file of more members is refused with exit 2. A report that only shows that a file just under that limit takes about 2 seconds is already known.

The time to read a `#Fields:` line, a price file or an edge map is in proportion to its size. The time of a price file with very many regions also grows with the number of months in the log (40000 regions and 1572 months took 13 seconds); that is described in the README, "Input rules and limits". A report that only shows that product is already known. A file that is slow in another way, or a small file that takes much longer than that, is a vulnerability.
