//! SIGINT stops the tool with exit 130 and never leaves a report that looks finished.
#![cfg(unix)]

use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A log with many distinct paths, so the report is far bigger than a pipe buffer.
fn big_log(label: &str) -> (Scratch, PathBuf) {
    let dir = std::env::temp_dir().join(format!("cdn-egress-bill-{}-{label}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut body = String::from("#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n");
    for i in 0..40_000 {
        body.push_str(&format!("2026-10-31\tIAD89-C2\t1000\t/path/{i}\thttps\n"));
    }
    let log = dir.join("big.log");
    fs::write(&log, body).unwrap();
    (Scratch(dir), log)
}

fn interrupt(pid: u32) {
    let status = Command::new("kill")
        .arg("-INT")
        .arg(pid.to_string())
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn sigint_while_the_report_is_blocked_on_a_full_pipe_exits_130() {
    let (_scratch, log) = big_log("sigint-pipe");
    let mut child = Command::new(env!("CARGO_BIN_EXE_cdn-egress-bill"))
        .args(["--as-of", "2026-11-20"])
        .arg(&log)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Nobody reads stdout, so the write of the big report blocks until the signal.
    std::thread::sleep(Duration::from_millis(1500));
    assert!(child.try_wait().unwrap().is_none(), "the tool ended early");
    interrupt(child.id());
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        assert!(started.elapsed() < Duration::from_secs(20), "no exit");
        std::thread::sleep(Duration::from_millis(20));
    };
    let mut err = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut err)
        .unwrap();
    assert_eq!(status.code(), Some(130), "{status:?} {err}");
    assert_eq!(err, "cdn-egress-bill: interrupted by SIGINT\n");
}
