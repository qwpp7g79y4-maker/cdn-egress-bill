//! `--output FILE` writes the report to a file in one rename and never half of it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cdn-egress-bill"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/twee_regios.log")
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "cdn-egress-bill-{}-out-{label}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(&self.0)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(extra: &[&str], output: &Path) -> Output {
    command()
        .args(extra)
        .arg("--output")
        .arg(output)
        .arg(fixture())
        .output()
        .unwrap()
}

#[test]
fn output_file_holds_exactly_the_report_that_stdout_would_have() {
    let dir = Scratch::new("same");
    for extra in [&[][..], &["--format", "json"], &["--md"]] {
        let target = dir.path("report");
        let written = run(extra, &target);
        assert_eq!(written.status.code(), Some(0), "{extra:?}");
        assert!(written.stdout.is_empty(), "{extra:?}");
        let plain = command().args(extra).arg(fixture()).output().unwrap();
        assert_eq!(fs::read(&target).unwrap(), plain.stdout, "{extra:?}");
        assert_eq!(dir.names(), ["report"], "no temporary file is left");
    }
}

#[test]
fn an_existing_file_is_replaced_by_the_whole_new_report() {
    let dir = Scratch::new("replace");
    let target = dir.path("report");
    fs::write(&target, "old\n".repeat(100_000)).unwrap();
    let out = run(&[], &target);
    assert_eq!(out.status.code(), Some(0));
    let plain = command().arg(fixture()).output().unwrap();
    assert_eq!(fs::read(&target).unwrap(), plain.stdout);
}

#[test]
fn a_missing_parent_directory_is_exit_2_and_is_not_created() {
    let dir = Scratch::new("missing");
    let target = dir.path("nope/report");
    let out = run(&[], &target);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("--output"), "{err}");
    assert!(err.contains("nope/report"), "{err}");
    assert!(!err.contains("panicked"), "{err}");
    assert_eq!(dir.names(), Vec::<String>::new());
}

#[test]
fn a_directory_as_output_is_refused_and_kept() {
    let dir = Scratch::new("isdir");
    let target = dir.path("sub");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("keep"), "x").unwrap();
    let out = run(&[], &target);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stderr).unwrap().contains("--output"));
    assert_eq!(fs::read_to_string(target.join("keep")).unwrap(), "x");
    assert_eq!(dir.names(), ["sub"]);
}

#[test]
fn a_failing_run_leaves_an_existing_output_file_alone() {
    let dir = Scratch::new("failing");
    let target = dir.path("report");
    fs::write(&target, "old report\n").unwrap();
    let out = command()
        .arg("--output")
        .arg(&target)
        .arg(dir.path("no-such-log"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&target).unwrap(), "old report\n");
    assert_eq!(dir.names(), ["report"]);
}

#[test]
fn strict_still_exits_3_and_writes_the_file() {
    let dir = Scratch::new("strict");
    let target = dir.path("report");
    let out = run(&["--strict"], &target);
    assert_eq!(out.status.code(), Some(3));
    assert!(out.stdout.is_empty());
    assert!(fs::read_to_string(&target).unwrap().contains("onbekend"));
}

#[test]
fn output_without_a_value_is_a_usage_error() {
    let out = command().arg(fixture()).arg("--output").output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[cfg(unix)]
#[test]
fn a_symlink_as_output_is_replaced_and_its_target_is_not_written() {
    let dir = Scratch::new("symlink");
    let victim = dir.path("victim");
    fs::write(&victim, "precious\n").unwrap();
    let link = dir.path("report");
    std::os::unix::fs::symlink(&victim, &link).unwrap();
    let out = run(&[], &link);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(fs::read_to_string(&victim).unwrap(), "precious\n");
    assert!(!fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    let plain = command().arg(fixture()).output().unwrap();
    assert_eq!(fs::read(&link).unwrap(), plain.stdout);
}

#[cfg(unix)]
#[test]
fn a_name_with_control_characters_is_escaped_in_the_error() {
    let dir = Scratch::new("escape");
    let bad = format!("{}/no\u{1b}[31m\nsuch/report", dir.0.display());
    let out = command()
        .args(["--as-of", "2026-11-20"])
        .arg("--output")
        .arg(&bad)
        .arg(fixture())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(!err.contains('\u{1b}'), "{err:?}");
    assert_eq!(err.matches('\n').count(), 1, "{err:?}");
}
