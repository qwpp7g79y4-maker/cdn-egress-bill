//! Every JSON report starts with `"schema_version": 1`. The tool gives no JSON on an error.

use std::path::PathBuf;
use std::process::{Command, Output};

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cdn-egress-bill"))
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

const HEAD: &str = "{\n  \"schema_version\": 1,\n  \"estimate\": true,\n";

fn json(extra: &[&str], log: &str) -> Output {
    command()
        .args(["--format", "json"])
        .args(extra)
        .arg(fixture(log))
        .output()
        .unwrap()
}

#[test]
fn every_json_report_has_schema_version_1_first() {
    let cases: [(&[&str], &str); 5] = [
        (&[], "twee_regios.log"),
        (&[], "documented_full.log"),
        (&[], "documented_header_only.log"),
        (&["--month", "2026-03"], "twee_regios.log"),
        (&["--month", "2030-01"], "twee_regios.log"),
    ];
    for (extra, log) in cases {
        let out = json(extra, log);
        assert_eq!(out.status.code(), Some(0), "{extra:?} {log}");
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.starts_with(HEAD), "{extra:?} {log}: {text}");
        assert_eq!(
            text.matches("\"schema_version\"").count(),
            1,
            "{extra:?} {log}"
        );
    }
}

#[test]
fn schema_version_is_also_there_with_strict_quiet_verbose_and_as_of() {
    for extra in [
        &["--strict"][..],
        &["--quiet"],
        &["--verbose"],
        &["--as-of", "2027-06-01"],
    ] {
        let out = json(extra, "twee_regios.log");
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.starts_with(HEAD), "{extra:?}: {text}");
    }
}

#[test]
fn schema_version_is_also_in_a_json_file_from_output() {
    let dir = std::env::temp_dir().join(format!("cdn-egress-bill-{}-schema", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let target = dir.join("report.json");
    let out = command()
        .args(["--format", "json", "--output"])
        .arg(&target)
        .arg(fixture("twee_regios.log"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = std::fs::read_to_string(&target).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(text.starts_with(HEAD), "{text}");
}

#[test]
fn an_error_with_json_selected_prints_no_json() {
    for args in [
        &["--format", "json", "--month", "2026-13"][..],
        &["--format", "json", "--as-of", "nonsense"],
        &["--format", "json", "--prices", "/nonexistent-prices"],
    ] {
        let out = command()
            .args(args)
            .arg(fixture("twee_regios.log"))
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
        assert!(!String::from_utf8(out.stderr)
            .unwrap()
            .contains("schema_version"));
    }
    let out = command()
        .args(["--format", "json", "/nonexistent-log"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
