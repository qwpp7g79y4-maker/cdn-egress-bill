//! The input shape of the CloudFront standard (legacy) access log as the documentation gives it:
//! <https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/standard-logging-legacy-s3.html>
//! (the "Log file format" section lists the 33 fields, a `#Version: 1.0` line, a `#Fields:` line,
//! tab separated rows, `-` for a field without a value, and `%20` style encoding in the values).
//! The fixtures were written by hand from that description; nothing was fetched.
use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn run(args: &[&str], logs: &[&str]) -> (i32, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cdn-egress-bill"));
    command.args(args);
    for log in logs {
        command.arg(fixture(log));
    }
    let output = command.output().unwrap();
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
    )
}

#[test]
fn the_full_documented_log_costs_exactly_one_gb_above_the_free_terabyte() {
    let (code, text) = run(
        &["--as-of", "2026-11-20", "--strict"],
        &["documented_full.log"],
    );
    assert_eq!(code, 0);
    // 1 TiB + 1000000000 + 73741824 bytes = 1 TiB + 1 GiB; the GiB above the free TiB is 0.085 USD.
    assert!(text.contains("\nmaand: 2026-10\n"));
    assert!(text.contains("\nestimated_usd_month=0.085000\n"));
    assert!(text.contains("\nUS/Mexico/Canada\tIAD\t1100585369600\t1\t2\t0\t0.085000\tja\n"));
    assert!(text.contains("\n/a.js\t73741824\t1\t0.000006\t0\t0\n"));
    assert!(text.contains("\n/img/logo.png\t1000000000\t1\t0.000077\t0\t0\n"));
    // A path with percent-encoding is reported as it is written in the log.
    assert!(text.contains("/video/a%20b/%E2%82%AC.mp4\t1099511627776\t1\t0.084917\t0\t0\n"));
    assert!(text.contains("\nonberekend_bytes: 0\n"));
}

#[test]
fn the_full_documented_log_in_json_has_the_same_amount_in_micro_usd() {
    let (code, text) = run(
        &["--as-of", "2026-11-20", "--format", "json"],
        &["documented_full.log"],
    );
    assert_eq!(code, 0);
    assert!(text.contains("\"estimated_usd_month_micros\": 85000,"));
}

#[test]
fn a_log_with_only_the_needed_fields_is_read_and_two_logs_add_up() {
    let (code, text) = run(&["--as-of", "2026-11-20"], &["documented_minimal.log"]);
    assert_eq!(code, 0);
    // 1 GiB, all of it inside the free terabyte.
    assert!(text.contains("\nestimated_usd_month=0.000000\n"));
    let (code, both) = run(
        &["--as-of", "2026-11-20"],
        &["documented_full.log", "documented_minimal.log"],
    );
    assert_eq!(code, 0);
    // 1 TiB + 2 GiB: two GiB above the free terabyte.
    assert!(both.contains("\nestimated_usd_month=0.170000\n"));
}

#[test]
fn a_log_with_a_header_and_no_rows_is_an_empty_report() {
    let (code, text) = run(
        &["--as-of", "2026-11-20", "--strict"],
        &["documented_header_only.log"],
    );
    assert_eq!(code, 0);
    assert!(text.contains("geen verkeer"));
}
