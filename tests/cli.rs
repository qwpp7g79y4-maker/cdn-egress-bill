use std::fs;
#[cfg(target_os = "linux")]
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
#[cfg(unix)]
use std::process::Stdio;

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cdn-egress-bill"))
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/twee_regios.log")
}

#[test]
fn help_lists_log_files() {
    let output = command().arg("--help").output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("standaardlogbestanden"));
    assert!(text.contains("offline"));
}

#[test]
fn version_is_1_0_0() {
    let output = command().arg("--version").output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("1.0.0"));
}

#[test]
fn missing_log_exits_2() {
    let output = command().output().unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn strict_exits_3_and_still_prints_onbekend() {
    let output = command().arg("--strict").arg(fixture()).output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("onbekend\tZZZ\t50\t0\t1\t0\t-\tnee"));
}

#[test]
fn json_format_on_the_fixture() {
    let output = command()
        .arg("--format")
        .arg("json")
        .arg(fixture())
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text
        .contains("\"source_url\": \"https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/\""));
    assert!(text.contains("\"/video.mp4\""));
    assert!(text.contains("\"/img.png\""));
    assert!(text.contains("\"checked_on\": \"2026-11-09\""));
}

#[test]
fn month_filter_keeps_march() {
    let march = command()
        .arg("--month")
        .arg("2026-03")
        .arg(fixture())
        .output()
        .unwrap();
    assert!(march.status.success());
    let text = String::from_utf8(march.stdout).unwrap();
    assert!(text.contains("maand: 2026-03"));
    assert!(text.contains("/video.mp4"));
    let april = command()
        .arg("--month")
        .arg("2026-04")
        .arg(fixture())
        .output()
        .unwrap();
    assert!(april.status.success());
    let empty = String::from_utf8(april.stdout).unwrap();
    assert!(empty.contains("geen verkeer"));
    assert!(!empty.contains("/video.mp4"));
}

#[test]
fn bad_month_exits_2() {
    let output = command()
        .arg("--month")
        .arg("2026-13")
        .arg(fixture())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
#[cfg(unix)]
fn unreadable_log_exits_2() {
    let output = command()
        .arg("/tmp/cdn-egress-bill-no-such-log")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("cdn-egress-bill-no-such-log"));
}

#[test]
fn clap_error_escapes_bidi_cr_and_newline_in_user_input() {
    let output = command()
        .arg("--format")
        .arg("x\u{202e}y\rz\nw")
        .arg(fixture())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(!err.contains('\u{202e}'), "{err:?}");
    assert!(!err.contains('\r'), "{err:?}");
    assert!(!err.contains("z\nw"), "{err:?}");
    assert!(err.contains("\\u202e"), "{err:?}");
}

#[test]
#[cfg(target_os = "linux")]
fn esc_in_filename_is_not_raw_on_stderr() {
    let path = std::env::temp_dir().join(format!(
        "cdn-egress-bill-{}-\u{1b}-missing.log",
        std::process::id()
    ));
    let output = command().arg(&path).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        !output.stderr.contains(&0x1bu8),
        "stderr bevat 0x1b: {:?}",
        output.stderr
    );
    let err = String::from_utf8(output.stderr).unwrap();
    let reason = std::fs::File::open(std::env::temp_dir().join(format!(
        "cdn-egress-bill-{}-no-such-reason.log",
        std::process::id()
    )))
    .unwrap_err()
    .to_string();
    assert!(err.contains(&reason), "{err}");
    assert!(err.contains("\\u001b"), "{err}");
}

#[test]
fn repeated_field_name_exits_2() {
    let path = std::env::temp_dir().join(format!(
        "cdn-egress-bill-{}-dup-field.log",
        std::process::id()
    ));
    fs::write(
        &path,
        "#Version: 1.0\n\
         #Fields: date x-edge-location sc-bytes sc-bytes cs-uri-stem cs-protocol\n\
         2026-03-01\tAMS1\t10\t99\t/a\thttps\n",
    )
    .unwrap();
    let output = command().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("sc-bytes"), "{err}");
    assert!(err.contains("dubbele"), "{err}");
    assert!(output.stdout.is_empty(), "{err}");

    let path = std::env::temp_dir().join(format!(
        "cdn-egress-bill-{}-two-names.log",
        std::process::id()
    ));
    fs::write(
        &path,
        "#Version: 1.0\n\
         #Fields: date time x-edge-location sc-bytes cs-uri-stem cs-protocol\n\
         2026-03-01\t12:00:00\tAMS1\t10\t/a\thttps\n",
    )
    .unwrap();
    let output = command().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.stdout.is_empty());
}

#[test]
fn overflowing_byte_count_names_the_log_and_exits_2() {
    let path = std::env::temp_dir().join(format!(
        "cdn-egress-bill-{}-overflow.log",
        std::process::id()
    ));
    let mut text = String::from("#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n");
    for _ in 0..300 {
        text.push_str("2026-03-01\tAMS1\t18446744073709551615\t/a\thttps\n");
    }
    fs::write(&path, text).unwrap();
    let output = command().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("cdn-egress-bill-"), "{err}");
    assert!(err.contains("overflow.log"), "{err}");
    assert!(err.contains("overflow"), "{err}");
    assert!(!err.contains("18446744073709551615"), "{err}");
    assert!(!err.contains("panicked"), "{err}");
    assert!(output.stdout.is_empty(), "{err}");
}

#[test]
fn invalid_utf8_log_exits_2() {
    let path =
        std::env::temp_dir().join(format!("cdn-egress-bill-{}-utf8.log", std::process::id()));
    fs::write(&path, b"#Fields: date\xff\n").unwrap();
    let output = command().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("valid UTF-8"), "{err}");
    assert!(output.stdout.is_empty(), "{err}");
}

#[test]
fn plus_date_exits_2() {
    let path =
        std::env::temp_dir().join(format!("cdn-egress-bill-{}-plus.log", std::process::id()));
    fs::write(
        &path,
        "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n\
         2026-03-02\tAMS1\t10\t/a\thttps\n\
         2026-+3-02\tAMS1\t12\t/b\thttps\n",
    )
    .unwrap();
    let output = command().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("2026-+3-02"), "{err}");
    assert!(output.stdout.is_empty(), "{err}");

    let month = command()
        .arg("--month")
        .arg("2026-+3")
        .arg(fixture())
        .output()
        .unwrap();
    assert_eq!(month.status.code(), Some(2));
    let month_err = String::from_utf8(month.stderr).unwrap();
    assert!(month_err.contains("2026-+3"), "{month_err}");
    assert!(month.stdout.is_empty(), "{month_err}");
}

#[cfg(unix)]
fn closed_writer() -> Stdio {
    let (stdout, _stderr) = shared_closed_pipe();
    stdout
}

#[cfg(unix)]
fn shared_closed_pipe() -> (Stdio, Stdio) {
    use std::os::unix::io::OwnedFd;
    use std::os::unix::net::UnixStream;
    let (reader, writer) = UnixStream::pair().unwrap();
    drop(reader);
    let writer = OwnedFd::from(writer);
    let stdout = Stdio::from(writer.try_clone().unwrap());
    let stderr = Stdio::from(writer);
    (stdout, stderr)
}

#[cfg(target_os = "linux")]
struct FewKbLog {
    path: PathBuf,
}

#[cfg(target_os = "linux")]
impl FewKbLog {
    fn create(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "cdn-egress-bill-{}-{label}.log",
            std::process::id()
        ));
        let mut body =
            String::from("#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n");
        let line = "2026-03-02\tAMS1\t11\t/a\thttps\n";
        while body.len() < 4 * 1024 {
            body.push_str(line);
        }
        assert!(
            (2 * 1024..8 * 1024).contains(&body.len()),
            "log is {} bytes",
            body.len()
        );
        fs::write(&path, body).unwrap();
        Self { path }
    }
}

#[cfg(target_os = "linux")]
impl Drop for FewKbLog {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[test]
#[cfg(unix)]
fn stdout_write_error_exits_2() {
    // Een gesloten leeskant geeft EPIPE, het geval `programma | head`.
    let log = fixture();
    let child = command()
        .arg(&log)
        .stdout(closed_writer())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("stdout"), "{err}");
    assert!(!err.contains("panicked"), "{err}");
}

#[test]
#[cfg(target_os = "linux")]
fn closed_or_full_stdout_exits_2_without_panic() {
    let log = FewKbLog::create("stdio");
    let ok = command()
        .args(["--as-of", "2026-11-20"])
        .arg(&log.path)
        .output()
        .unwrap();
    assert!(
        ok.status.success(),
        "{}",
        String::from_utf8_lossy(&ok.stderr)
    );
    assert!(!ok.stdout.is_empty());

    let discarded = command()
        .args(["--as-of", "2026-11-20"])
        .arg(&log.path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert!(
        discarded.status.success(),
        "{}",
        String::from_utf8_lossy(&discarded.stderr)
    );
    assert!(discarded.stderr.is_empty());

    let (stdout, stderr) = shared_closed_pipe();
    let child = command()
        .args(["--as-of", "2026-11-20"])
        .arg(&log.path)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));

    let closed = Command::new("sh")
        .arg("-c")
        .arg("exec \"$0\" \"$1\" >&-")
        .arg(env!("CARGO_BIN_EXE_cdn-egress-bill"))
        .arg(&log.path)
        .output()
        .unwrap();
    assert_eq!(closed.status.code(), Some(2));
    let err = String::from_utf8(closed.stderr).unwrap();
    assert!(err.contains("stdout"), "{err}");
    assert!(!err.contains("panicked"), "{err}");
    assert!(closed.stdout.is_empty(), "{err}");

    let read_only = fs::File::open(&log.path).unwrap();
    let child = command()
        .arg(&log.path)
        .stdout(Stdio::from(read_only))
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("stdout"), "{err}");
    assert!(!err.contains("panicked"), "{err}");
    assert!(output.stdout.is_empty(), "{err}");

    if Path::new("/dev/full").exists() {
        let out = fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        let err_file = fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        let child = command()
            .arg(&log.path)
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err_file))
            .spawn()
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(2));
    }
}

#[test]
#[cfg(unix)]
fn closed_stderr_keeps_exit_2_or_3() {
    let missing = command()
        .arg("/tmp/cdn-egress-bill-no-such-log")
        .stdout(Stdio::null())
        .stderr(closed_writer())
        .spawn()
        .unwrap();
    let output = missing.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));

    let strict = command()
        .arg("--strict")
        .arg(fixture())
        .stdout(Stdio::null())
        .stderr(closed_writer())
        .spawn()
        .unwrap();
    let output = strict.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(3));

    #[cfg(target_os = "linux")]
    if Path::new("/dev/full").exists() {
        let err_file = fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        let missing = command()
            .arg("/tmp/cdn-egress-bill-no-such-log")
            .stdout(Stdio::null())
            .stderr(Stdio::from(err_file))
            .spawn()
            .unwrap();
        assert_eq!(missing.wait_with_output().unwrap().status.code(), Some(2));

        let err_file = fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        let strict = command()
            .arg("--strict")
            .arg(fixture())
            .stdout(Stdio::null())
            .stderr(Stdio::from(err_file))
            .spawn()
            .unwrap();
        assert_eq!(strict.wait_with_output().unwrap().status.code(), Some(3));
    }
}

#[test]
fn prices_directory_exits_2() {
    let output = command()
        .arg("--prices")
        .arg(std::env::temp_dir())
        .arg(fixture())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("geen gewoon bestand"), "{err}");
    assert!(output.stdout.is_empty(), "{err}");
}

#[test]
#[cfg(unix)]
fn prices_fifo_exits_2_without_blocking() {
    let path = std::env::temp_dir().join(format!(
        "cdn-egress-bill-{}-prices.fifo",
        std::process::id()
    ));
    let _ = fs::remove_file(&path);
    let made = Command::new("mkfifo").arg(&path).status().unwrap();
    assert!(made.success());
    // Geen schrijver: openen zou blokkeren, dus moet de stat-controle weigeren.
    let output = command()
        .arg("--prices")
        .arg(&path)
        .arg(fixture())
        .output()
        .unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("geen gewoon bestand"), "{err}");
}

#[test]
#[cfg(unix)]
fn log_fifo_exits_2_without_blocking() {
    let path =
        std::env::temp_dir().join(format!("cdn-egress-bill-{}-log.fifo", std::process::id()));
    let _ = fs::remove_file(&path);
    let made = Command::new("mkfifo").arg(&path).status().unwrap();
    assert!(made.success());
    let mut child = command()
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let start = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if start.elapsed() > std::time::Duration::from_secs(5) {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    let _ = fs::remove_file(&path);
    let status = status.expect("tool hangt op een fifo zonder schrijver");
    assert_eq!(status.code(), Some(2));
    let mut err = String::new();
    std::io::Read::read_to_string(&mut child.stderr.take().unwrap(), &mut err).unwrap();
    assert!(err.contains("geen gewoon bestand"), "{err}");
}

#[test]
#[cfg(target_os = "linux")]
fn log_through_pipe_to_dev_stdin_counts_like_the_file() {
    use std::io::Write;
    let bytes = fs::read(fixture()).unwrap();
    let from_file = command().arg(fixture()).output().unwrap();
    assert_eq!(from_file.status.code(), Some(0));
    let mut child = command()
        .arg("/dev/stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(&bytes));
    let piped = child.wait_with_output().unwrap();
    writer.join().unwrap().unwrap();
    let err = String::from_utf8_lossy(&piped.stderr);
    assert_eq!(piped.status.code(), Some(0), "{err}");
    assert_eq!(piped.stdout, from_file.stdout);
}

#[test]
#[cfg(target_os = "linux")]
fn stdout_closed_full_and_read_write_null_are_told_apart() {
    let bin = env!("CARGO_BIN_EXE_cdn-egress-bill");
    let log = fixture();
    let run = |script: &str, strict: bool| {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(script).arg(bin).arg(&log);
        if strict {
            cmd.arg("--strict");
        }
        cmd.output().unwrap()
    };
    // 1<>/dev/null is een gewone, lees-schrijf stdout: niet "gesloten".
    let rw_strict = run(
        "exec \"$0\" --as-of 2026-11-20 --strict \"$1\" 1<>/dev/null",
        false,
    );
    assert_eq!(rw_strict.status.code(), Some(3));
    let rw = run("exec \"$0\" --as-of 2026-11-20 \"$1\" 1<>/dev/null", false);
    assert_eq!(rw.status.code(), Some(0));
    assert!(rw.stderr.is_empty());
    // Echt gesloten stdout is een schrijffout.
    let closed = run("exec \"$0\" --as-of 2026-11-20 \"$1\" >&-", false);
    assert_eq!(closed.status.code(), Some(2));
    let err = String::from_utf8(closed.stderr).unwrap();
    assert!(err.contains("stdout") && !err.contains("panicked"), "{err}");
    // /dev/full is een schrijffout.
    if Path::new("/dev/full").exists() {
        let full = run("exec \"$0\" --as-of 2026-11-20 \"$1\" >/dev/full", false);
        assert_eq!(full.status.code(), Some(2));
        let err = String::from_utf8(full.stderr).unwrap();
        assert!(!err.contains("panicked"), "{err}");
    }
}

#[test]
#[cfg(target_os = "linux")]
fn help_and_version_to_closed_or_full_stdout_exit_2() {
    for flag in ["--help", "--version"] {
        let closed = Command::new("sh")
            .arg("-c")
            .arg("exec \"$0\" \"$1\" >&-")
            .arg(env!("CARGO_BIN_EXE_cdn-egress-bill"))
            .arg(flag)
            .output()
            .unwrap();
        let err = String::from_utf8(closed.stderr).unwrap();
        assert_eq!(closed.status.code(), Some(2), "{flag}: {err}");
        assert!(err.contains("stdout"), "{flag}: {err}");
        assert!(!err.contains("panicked"), "{flag}: {err}");
        assert!(closed.stdout.is_empty(), "{flag}: {err}");

        let out = fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        let full = command()
            .arg(flag)
            .stdout(Stdio::from(out))
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        let err = String::from_utf8(full.stderr).unwrap();
        assert_eq!(full.status.code(), Some(2), "{flag}: {err}");
        assert!(err.contains("stdout"), "{flag}: {err}");
        assert!(!err.contains("panicked"), "{flag}: {err}");
    }
}

#[test]
fn same_log_twice_counts_twice() {
    let path =
        std::env::temp_dir().join(format!("cdn-egress-bill-{}-twice.log", std::process::id()));
    fs::write(
        &path,
        "#Version: 1.0\n\
         #Fields: date time x-edge-location sc-bytes cs-uri-stem cs-protocol\n\
         2026-03-01\t12:00:00\tAMS1\t1000\t/a\thttps\n",
    )
    .unwrap();
    let once = command()
        .arg("--format")
        .arg("json")
        .arg(&path)
        .output()
        .unwrap();
    let twice = command()
        .arg("--format")
        .arg("json")
        .arg(&path)
        .arg(&path)
        .output()
        .unwrap();
    let _ = fs::remove_file(&path);
    assert!(once.status.success());
    assert!(twice.status.success());
    let once = String::from_utf8(once.stdout).unwrap();
    let twice = String::from_utf8(twice.stdout).unwrap();
    assert!(once.contains("\"bytes\": 1000,"), "{once}");
    assert!(twice.contains("\"bytes\": 2000,"), "{twice}");
    assert!(twice.contains("\"https_requests\": 2,"), "{twice}");
}

#[test]
#[cfg(unix)]
fn oversized_regular_file_exits_2_before_reading() {
    let path = std::env::temp_dir().join(format!("cdn-egress-bill-{}-big.log", std::process::id()));
    let file = fs::File::create(&path).unwrap();
    file.set_len(268_435_457).unwrap();
    drop(file);
    let shown = path.display().to_string();
    let runs: [Vec<std::ffi::OsString>; 3] = [
        vec![path.clone().into()],
        vec!["--prices".into(), path.clone().into(), fixture().into()],
        vec!["--edges".into(), path.clone().into(), fixture().into()],
    ];
    let outputs: Vec<std::process::Output> = runs
        .into_iter()
        .map(|args| command().args(args).output().unwrap())
        .collect();
    let _ = fs::remove_file(&path);
    for output in outputs {
        assert_eq!(output.status.code(), Some(2));
        let err = String::from_utf8(output.stderr).unwrap();
        assert!(
            err.contains(&format!("{shown}: file is larger than 268435456 bytes")),
            "{err}"
        );
    }
}

#[test]
#[cfg(target_os = "linux")]
fn oversized_pipe_to_dev_stdin_exits_2_without_hanging() {
    use std::io::Write;
    let mut child = command()
        .arg("/dev/stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    // Offers one byte more than the limit; the writer ends on a broken pipe.
    let writer = std::thread::spawn(move || {
        let block = vec![b'a'; 1 << 20];
        let mut sent_all = true;
        for _ in 0..256 {
            if stdin.write_all(&block).is_err() {
                sent_all = false;
                break;
            }
        }
        if sent_all {
            let _ = stdin.write_all(b"a");
        }
    });
    let reader = std::thread::spawn(move || {
        let mut err = String::new();
        let _ = std::io::Read::read_to_string(&mut stderr, &mut err);
        err
    });
    let start = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if start.elapsed() > std::time::Duration::from_secs(8) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("oversized pipe made the program hang");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    let _ = writer.join();
    let err = reader.join().unwrap();
    assert_eq!(status.code(), Some(2), "{err}");
    assert!(
        err.contains("/dev/stdin: file is larger than 268435456 bytes"),
        "{err}"
    );
}

fn temp_file(label: &str, text: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("cdn-egress-bill-{}-{label}", std::process::id()));
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn custom_edges_file_prices_every_edge_so_strict_exits_0() {
    let edges = temp_file(
        "all.edges",
        "AMS europe_israel_turkiye\nFRA europe_israel_turkiye\nIAD us_mexico_canada\nZZZ us_mexico_canada\n",
    );
    let output = command()
        .arg("--strict")
        .arg("--edges")
        .arg(&edges)
        .arg(fixture())
        .output()
        .unwrap();
    let _ = fs::remove_file(&edges);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(!text.contains("onbekend"), "{text}");
}

#[test]
fn edges_file_with_unknown_price_region_exits_2() {
    let edges = temp_file("bad-region.edges", "AMS no_such_region\n");
    let output = command()
        .arg("--edges")
        .arg(&edges)
        .arg(fixture())
        .output()
        .unwrap();
    let _ = fs::remove_file(&edges);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(
        err.contains("edge AMS hoort bij onbekende prijsregio no_such_region"),
        "{err}"
    );
}

#[test]
fn custom_prices_file_is_used_for_the_report() {
    let text = include_str!("../data/prices.txt").replace("2026-11-09", "2026-11-10");
    let prices = temp_file("custom.prices", &text);
    let output = command()
        .arg("--format")
        .arg("json")
        .arg("--prices")
        .arg(&prices)
        .arg(fixture())
        .output()
        .unwrap();
    let _ = fs::remove_file(&prices);
    assert_eq!(output.status.code(), Some(0));
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("\"checked_on\": \"2026-11-10\""), "{json}");
    assert!(!json.contains("2026-11-09"), "{json}");
}

#[test]
fn stem_with_mongolian_vowel_separator_or_deprecated_format_char_is_escaped() {
    for (label, ch, esc) in [
        ("u180e", '\u{180e}', "\\u180e"),
        ("u206a", '\u{206a}', "\\u206a"),
    ] {
        let log = temp_file(
            &format!("{label}.log"),
            &format!(
                "#Version: 1.0\n\
                 #Fields: date time x-edge-location sc-bytes cs-uri-stem cs-protocol\n\
                 2026-03-01\t12:00:00\tAMS1\t1000\t/a{ch}b\thttps\n"
            ),
        );
        let text = command().arg(&log).output().unwrap();
        let json = command()
            .arg("--format")
            .arg("json")
            .arg(&log)
            .output()
            .unwrap();
        let _ = fs::remove_file(&log);
        assert!(text.status.success());
        let text = String::from_utf8(text.stdout).unwrap();
        let json = String::from_utf8(json.stdout).unwrap();
        assert!(!text.contains(ch), "{text}");
        assert!(text.contains(esc), "{text}");
        assert!(json.contains("\"bytes\": 1000,"), "{json}");
    }
}

#[test]
fn stem_with_hangul_choseong_filler_is_escaped_and_bytes_still_count() {
    let log = temp_file(
        "u115f.log",
        "#Version: 1.0\n\
         #Fields: date time x-edge-location sc-bytes cs-uri-stem cs-protocol\n\
         2026-03-01\t12:00:00\tAMS1\t1000\t/a\u{115f}b\thttps\n",
    );
    let text = command().arg(&log).output().unwrap();
    let json = command()
        .arg("--format")
        .arg("json")
        .arg(&log)
        .output()
        .unwrap();
    let _ = fs::remove_file(&log);
    assert!(text.status.success());
    let text = String::from_utf8(text.stdout).unwrap();
    let json = String::from_utf8(json.stdout).unwrap();
    assert!(!text.contains('\u{115f}'), "{text}");
    assert!(text.contains("\\u115f"), "{text}");
    assert!(!json.contains('\u{115f}'), "{json}");
    assert!(json.contains("\"bytes\": 1000,"), "{json}");
}

#[cfg(target_os = "linux")]
fn ff_named(label: &str) -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    let mut name = format!("cdn-egress-bill-{}-{label}-", std::process::id()).into_bytes();
    name.push(0xff);
    name.extend_from_slice(b".log");
    std::env::temp_dir().join(std::ffi::OsStr::from_bytes(&name))
}

#[test]
#[cfg(target_os = "linux")]
fn invalid_utf8_name_and_content_shows_byte_escape_not_replacement_char() {
    let path = ff_named("bad");
    fs::write(&path, b"#Fields: date\xff\n").unwrap();
    let output = command().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("\\xff.log"), "{err}");
    assert!(!err.contains('\u{fffd}'), "{err}");
    assert!(output.stdout.is_empty(), "{err}");
}

#[test]
#[cfg(target_os = "linux")]
fn valid_log_with_invalid_utf8_name_still_counts() {
    let path = ff_named("good");
    fs::copy(fixture(), &path).unwrap();
    let output = command().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);
    let plain = command().arg(fixture()).output().unwrap();
    assert_eq!(output.status.code(), plain.status.code());
    assert_eq!(output.stdout, plain.stdout);
    assert!(!output.stdout.is_empty());
}

#[cfg(unix)]
#[test]
fn bidi_character_in_file_name_is_escaped_in_errors_but_log_still_counts() {
    let dir = std::env::temp_dir().join(format!("cdn-egress-bill-{}-bidi", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    let odd = dir.join("a\u{202e}b.log");

    let missing = command().arg(&odd).output().unwrap();
    let err = String::from_utf8(missing.stderr).unwrap();
    assert_ne!(missing.status.code(), Some(0));
    assert!(!err.contains('\u{202e}'), "{err}");
    assert!(err.contains("a\\u202eb.log"), "{err}");

    fs::copy(fixture(), &odd).unwrap();
    let plain = command().arg(fixture()).output().unwrap();
    let named = command().arg(&odd).output().unwrap();
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(named.status.code(), plain.status.code());
    let named_out = String::from_utf8(named.stdout).unwrap();
    let plain_out = String::from_utf8(plain.stdout).unwrap();
    assert!(!named_out.contains('\u{202e}'), "{named_out}");
    assert_eq!(named_out.lines().last(), plain_out.lines().last());
}

fn temp_bytes(label: &str, bytes: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!("cdn-egress-bill-{}-{label}", std::process::id()));
    fs::write(&path, bytes).unwrap();
    path
}

fn gzip_member(bytes: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

const SMALL_LOG: &str = "#Version: 1.0\n\
     #Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n\
     2026-03-01\tAMS50-C1\t1000\t/caf\u{e9}\thttps\n\
     2026-03-02\tIAD89-P1\t2000\t/img.png\thttp\n";

fn stderr_of(args: &[&std::ffi::OsStr]) -> String {
    let output = command().args(args).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    String::from_utf8(output.stderr).unwrap()
}

/// A clap error must stay as long as clap's own text, whatever the argument holds.
#[test]
fn clap_error_with_an_escape_sequence_and_a_line_break_stays_on_one_line() {
    use std::ffi::OsStr;
    let log = fixture();
    let log: &OsStr = log.as_os_str();
    let plain_unknown = stderr_of(&[OsStr::new("--xFAKE"), log]);
    let plain_value = stderr_of(&[OsStr::new("--format"), OsStr::new("jFAKE"), log]);
    let cases = [
        ("\u{1b}[31m\nFAKE: line", "\\u000a"),
        ("\u{1b}[31m\rFAKE: line", "\\u000d"),
        ("\u{1b}[0m\u{202e}\nFAKE: line", "\\u202e"),
        ("\u{1b}[31m\u{2028}FAKE: line", "\\u2028"),
    ];
    for (tail, shown) in cases {
        let unknown_arg = format!("--x{tail}");
        let value_arg = format!("j{tail}");
        let unknown = stderr_of(&[OsStr::new(&unknown_arg), log]);
        let value = stderr_of(&[OsStr::new("--format"), OsStr::new(&value_arg), log]);
        for (err, plain) in [(&unknown, &plain_unknown), (&value, &plain_value)] {
            assert!(!err.contains('\u{1b}'), "{err:?}");
            assert!(!err.contains('\r'), "{err:?}");
            assert!(!err.contains('\u{202e}'), "{err:?}");
            assert!(!err.contains('\u{2028}'), "{err:?}");
            assert!(err.lines().all(|line| !line.starts_with("FAKE")), "{err:?}");
            assert_eq!(err.lines().count(), plain.lines().count(), "{err:?}");
            assert!(err.contains(shown), "{shown}: {err:?}");
        }
    }
}

/// An argument that is not UTF-8 is still an argument (a shell, `find -exec` or a directory
/// listing hands one over); a line break in it must not make a line of its own.
#[cfg(unix)]
#[test]
fn clap_error_with_a_non_utf8_argument_and_a_line_break_stays_on_one_line() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let log = fixture();
    let log: &OsStr = log.as_os_str();
    let plain_unknown = stderr_of(&[OsStr::new("--xFAKE"), log]);
    let plain_value = stderr_of(&[OsStr::new("--format"), OsStr::new("jFAKE"), log]);
    let plain_flag = stderr_of(&[OsStr::new("--strict=FAKE"), log]);
    let tails: [&[u8]; 3] = [
        b"\xff\nFAKE: line",
        b"\xff\x1b[31m\nFAKE: line",
        b"\xc3\r\nFAKE: line",
    ];
    for tail in tails {
        let with = |head: &[u8]| {
            let mut bytes = head.to_vec();
            bytes.extend_from_slice(tail);
            std::ffi::OsString::from(OsStr::from_bytes(&bytes))
        };
        let unknown = stderr_of(&[with(b"--x").as_os_str(), log]);
        let value = stderr_of(&[OsStr::new("--format"), with(b"j").as_os_str(), log]);
        let flag = stderr_of(&[with(b"--strict=").as_os_str(), log]);
        let short = stderr_of(&[with(b"-").as_os_str(), log]);
        for (err, plain) in [
            (&unknown, Some(&plain_unknown)),
            (&value, Some(&plain_value)),
            (&flag, Some(&plain_flag)),
            (&short, None),
        ] {
            assert!(!err.contains('\u{1b}'), "{err:?}");
            assert!(!err.contains('\r'), "{err:?}");
            assert!(err.lines().all(|line| !line.starts_with("FAKE")), "{err:?}");
            if let Some(plain) = plain {
                assert_eq!(err.lines().count(), plain.lines().count(), "{err:?}");
            }
        }
    }
}

/// The message for a non-UTF-8 argument names the bytes and keeps clap's own line layout
/// (no literal `\u000a` in place of clap's line breaks).
#[cfg(unix)]
#[test]
fn non_utf8_month_value_keeps_clap_lines_and_non_utf8_option_shows_its_bytes() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let log = fixture();
    let log: &OsStr = log.as_os_str();
    let plain = stderr_of(&[OsStr::new("--format"), OsStr::new("jFAKE"), log]);
    for value in [&b"\xff"[..], &b"2026-03\xff"[..], &b"\xff\nFAKE: line"[..]] {
        let err = stderr_of(&[OsStr::new("--month"), OsStr::from_bytes(value), log]);
        assert!(!err.contains("\\u000a"), "{err:?}");
        assert!(err.lines().all(|line| !line.starts_with("FAKE")), "{err:?}");
        assert_eq!(err.lines().count(), plain.lines().count() + 1, "{err:?}");
        assert!(err.starts_with("error: invalid UTF-8"), "{err:?}");
        assert!(err.contains("Usage: "), "{err:?}");
        assert!(err.ends_with("'--help'.\n"), "{err:?}");
    }
    let err = stderr_of(&[OsStr::from_bytes(b"--x\xff\nFAKE: line"), log]);
    assert!(err.contains("--x\\xff"), "{err:?}");
    let err = stderr_of(&[
        OsStr::new("--format"),
        OsStr::from_bytes(b"j\xff\nFAKE: line"),
        log,
    ]);
    assert!(err.contains("j\\xff"), "{err:?}");
}

#[test]
fn clap_error_ends_with_one_newline_and_no_blank_line() {
    let output = command().output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.ends_with("'--help'.\n"), "{err:?}");
}

#[test]
fn gzip_members_may_split_a_utf8_character() {
    let bytes = SMALL_LOG.as_bytes();
    let cut = SMALL_LOG.find('\u{e9}').unwrap() + 1;
    let whole = temp_bytes("whole.gz", &gzip_member(bytes));
    let split = temp_bytes(
        "split.gz",
        &[gzip_member(&bytes[..cut]), gzip_member(&bytes[cut..])].concat(),
    );
    let from_whole = command().arg(&whole).output().unwrap();
    let from_split = command().arg(&split).output().unwrap();
    let _ = fs::remove_file(&whole);
    let _ = fs::remove_file(&split);
    let err = String::from_utf8_lossy(&from_split.stderr);
    assert_eq!(from_split.status.code(), Some(0), "{err}");
    assert_eq!(from_split.stdout, from_whole.stdout);
    assert!(String::from_utf8(from_split.stdout)
        .unwrap()
        .contains("/caf\u{e9}"));
}

#[test]
fn gzip_is_recognised_by_its_first_bytes_and_by_an_upper_case_name() {
    let member = gzip_member(SMALL_LOG.as_bytes());
    let plain = temp_bytes("plain.log", SMALL_LOG.as_bytes());
    let bare = temp_bytes("magic-no-extension", &member);
    let upper = temp_bytes("upper.GZ", &member);
    let from_plain = command().arg(&plain).output().unwrap();
    let from_bare = command().arg(&bare).output().unwrap();
    let from_upper = command().arg(&upper).output().unwrap();
    for path in [&plain, &bare, &upper] {
        let _ = fs::remove_file(path);
    }
    assert_eq!(from_plain.status.code(), Some(0));
    assert_eq!(from_bare.status.code(), Some(0));
    assert_eq!(from_upper.status.code(), Some(0));
    assert_eq!(from_bare.stdout, from_plain.stdout);
    assert_eq!(from_upper.stdout, from_plain.stdout);
}

#[test]
fn the_order_of_the_log_lines_does_not_change_the_report() {
    let mut lines: Vec<&str> = SMALL_LOG.lines().collect();
    let header: Vec<&str> = lines.drain(..2).collect();
    let forward = format!("{}\n{}\n", header.join("\n"), lines.join("\n"));
    lines.reverse();
    let backward = format!("{}\n{}\n", header.join("\n"), lines.join("\n"));
    let first = temp_bytes("order-a.log", forward.as_bytes());
    let second = temp_bytes("order-b.log", backward.as_bytes());
    let a = command().arg(&first).output().unwrap();
    let b = command().arg(&second).output().unwrap();
    let _ = fs::remove_file(&first);
    let _ = fs::remove_file(&second);
    assert_eq!(a.status.code(), Some(0));
    assert!(!a.stdout.is_empty());
    assert_eq!(a.stdout, b.stdout);
}

/// The README quick start log and the report printed under it are the files in `examples/`,
/// and the report is what the binary prints.
#[test]
fn quick_start_log_and_report_match_the_examples_and_the_binary() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).unwrap();
    let log = fs::read_to_string(root.join("examples/access.log")).unwrap();
    let report = fs::read_to_string(root.join("examples/access.report.txt")).unwrap();

    let printf = readme
        .lines()
        .find(|line| line.starts_with("printf '"))
        .expect("README has the printf line");
    let body = printf
        .strip_prefix("printf '")
        .and_then(|rest| rest.strip_suffix("' > access.log"))
        .expect("printf line shape");
    assert_eq!(body.replace("\\n", "\n").replace("\\t", "\t"), log);

    let block = readme
        .split("```\nbron: ")
        .nth(1)
        .and_then(|rest| rest.split("\n```").next())
        .expect("README report block");
    assert_eq!(format!("bron: {block}\n"), report);

    let output = command()
        .arg(root.join("examples/access.log"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), report);
}

const LOG_HEADER: &str = "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n";

fn run_log(label: &str, body: &str) -> std::process::Output {
    let path = temp_bytes(label, format!("{LOG_HEADER}{body}").as_bytes());
    let output = command().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);
    output
}

#[test]
fn dates_and_months_are_accepted_from_1970_to_2100_only() {
    for date in ["1970-01-01", "2100-12-31"] {
        let output = run_log("date-ok.log", &format!("{date}\tAMS1\t1\t/a\thttps\n"));
        assert_eq!(output.status.code(), Some(0), "{date}");
    }
    for date in ["1969-12-31", "2101-01-01", "0000-01-01", "9999-12-31"] {
        let output = run_log("date-bad.log", &format!("{date}\tAMS1\t1\t/a\thttps\n"));
        assert_eq!(output.status.code(), Some(2), "{date}");
        let err = String::from_utf8(output.stderr).unwrap();
        assert!(err.contains(&format!("datum {date} is ongeldig")), "{err}");
    }
    for (month, code) in [
        ("1970-01", 0),
        ("2100-12", 0),
        ("1969-12", 2),
        ("2101-01", 2),
    ] {
        let output = command()
            .arg("--month")
            .arg(month)
            .arg(fixture())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(code), "{month}");
    }
}

/// The calendar rules at the edge of what is accepted: the century rule, month 00, month 13.
#[test]
fn leap_days_and_month_numbers_follow_the_calendar() {
    for (date, code) in [
        ("2024-02-29", 0),
        ("2000-02-29", 0),
        ("2100-02-28", 0),
        ("2100-02-29", 2),
        ("2026-02-29", 2),
        ("2026-04-31", 2),
        ("2026-00-10", 2),
        ("2026-13-01", 2),
    ] {
        let output = run_log("calendar.log", &format!("{date}\tAMS1\t1\t/a\thttps\n"));
        assert_eq!(output.status.code(), Some(code), "{date}");
    }
    for month in ["2026-00", "2026-13", "2026-3", ""] {
        let output = command()
            .arg("--month")
            .arg(month)
            .arg(fixture())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{month:?}");
    }
}

/// One empty field after the last tab is tolerated; a second one is a field too many.
#[test]
fn a_single_trailing_tab_on_a_data_row_is_tolerated() {
    let one = run_log("tab-one.log", "2026-03-01\tAMS1\t7\t/a\thttps\t\n");
    assert_eq!(one.status.code(), Some(0));
    assert!(String::from_utf8(one.stdout).unwrap().contains("/a\t7\t"));
    let two = run_log("tab-two.log", "2026-03-01\tAMS1\t7\t/a\thttps\t\t\n");
    assert_eq!(two.status.code(), Some(2));
    let err = String::from_utf8(two.stderr).unwrap();
    assert!(err.contains(":2: 6 velden, #Fields heeft 5"), "{err}");
}

/// The limit is "larger than 268435456 bytes": a file of exactly that size is read.
#[test]
#[cfg(unix)]
fn a_file_of_exactly_the_size_limit_is_read_and_one_byte_more_is_refused() {
    let path =
        std::env::temp_dir().join(format!("cdn-egress-bill-{}-edge.log", std::process::id()));
    let file = fs::File::create(&path).unwrap();
    file.set_len(268_435_456).unwrap();
    drop(file);
    let exact = command().arg(&path).output().unwrap();
    let err = String::from_utf8_lossy(&exact.stderr).into_owned();
    let _ = fs::remove_file(&path);
    assert_eq!(exact.status.code(), Some(2));
    assert!(err.contains("geen #Fields vóór de data"), "{err}");
    assert!(!err.contains("file is larger"), "{err}");
}

#[test]
fn sc_bytes_is_an_unsigned_64_bit_number_and_a_sum_that_does_not_fit_is_an_error() {
    let max = run_log(
        "u64-max.log",
        "2026-03-01\tAMS1\t18446744073709551615\t/a\thttps\n",
    );
    assert_eq!(max.status.code(), Some(0));
    for bad in ["18446744073709551616", "+5", "-5", "1.5", "0x10", ""] {
        let output = run_log(
            "u64-bad.log",
            &format!("2026-03-01\tAMS1\t{bad}\t/a\thttps\n"),
        );
        assert_eq!(output.status.code(), Some(2), "{bad:?}");
        let err = String::from_utf8(output.stderr).unwrap();
        assert!(err.contains(":2: sc-bytes is geen getal"), "{err}");
    }
    let many = "2026-03-01\tAMS1\t18446744073709551615\t/a\thttps\n".repeat(300);
    let output = run_log("u64-sum.log", &many);
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("u64-sum.log: overflow"), "{err}");
    assert!(output.stdout.is_empty());
}

#[test]
fn an_empty_log_and_a_header_only_log_succeed_even_with_strict() {
    for (label, body) in [("empty.log", ""), ("header-only.log", LOG_HEADER)] {
        let path = temp_bytes(label, body.as_bytes());
        let outputs: Vec<(bool, std::process::Output)> = [false, true]
            .into_iter()
            .map(|strict| {
                let mut run = command();
                if strict {
                    run.arg("--strict");
                }
                (strict, run.arg(&path).output().unwrap())
            })
            .collect();
        let _ = fs::remove_file(&path);
        for (strict, output) in outputs {
            assert_eq!(output.status.code(), Some(0), "{label} strict={strict}");
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(text.contains("geen verkeer"), "{text}");
        }
    }
}

#[test]
fn a_hash_line_that_holds_a_tab_is_a_data_row() {
    let output = run_log("hash-tab.log", "#Version: x\ty\n");
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains(":2: 2 velden, #Fields heeft 5"), "{err}");
    let no_tab = run_log("hash-no-tab.log", "#Version: x y\n");
    assert_eq!(no_tab.status.code(), Some(0));
}

/// Run the binary and give up after `seconds`. A child that is too slow is
/// killed and reaped, and `None` says so.
fn run_within(path_args: &[&std::ffi::OsStr], seconds: u64) -> Option<std::process::Output> {
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    let mut child = command()
        .args(path_args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        if child.try_wait().unwrap().is_some() {
            return Some(child.wait_with_output().unwrap());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Time limit for the work-limit tests. The fixed program needs well under a
/// second in a debug build; the quadratic one needs minutes.
const WORK_LIMIT_SECONDS: u64 = 20;

#[test]
fn a_fields_line_with_many_names_is_read_in_linear_time() {
    let mut header = String::from("#Fields:");
    for index in 0..200_000 {
        header.push_str(&format!(" f{index}"));
    }
    header.push_str(" date x-edge-location sc-bytes cs-uri-stem cs-protocol\n");
    let path = temp_bytes("many-fields.log", header.as_bytes());
    let done = run_within(&[path.as_os_str()], WORK_LIMIT_SECONDS);
    let mut twice = header.trim_end().to_string();
    twice.push_str(" f0\n");
    let dup = temp_bytes("many-fields-dup.log", twice.as_bytes());
    let dup_done = run_within(&[dup.as_os_str()], WORK_LIMIT_SECONDS);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&dup);
    let output = done.expect("a #Fields line of 200000 names must not take 20 seconds");
    assert_eq!(output.status.code(), Some(0));
    let output = dup_done.expect("a repeated name at the end must not take 20 seconds");
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains(":1: #Fields dubbele naam f0"), "{err}");
}

#[test]
fn a_price_file_with_many_regions_and_labels_is_read_in_linear_time() {
    let mut text =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/prices.txt"))
            .unwrap();
    for index in 0..100_000 {
        text.push_str(&format!("region r{index} 1 1 1 1 1 1 1 1 1\n"));
        text.push_str(&format!("label r{index} Region {index}\n"));
    }
    let prices = temp_bytes("many-regions.txt", text.as_bytes());
    let log = temp_bytes("many-regions.log", LOG_HEADER.as_bytes());
    let done = run_within(
        &[
            std::ffi::OsStr::new("--prices"),
            prices.as_os_str(),
            log.as_os_str(),
        ],
        WORK_LIMIT_SECONDS,
    );
    text.push_str("region r0 1 1 1 1 1 1 1 1 1\n");
    let dup_prices = temp_bytes("many-regions-dup.txt", text.as_bytes());
    let dup_done = run_within(
        &[
            std::ffi::OsStr::new("--prices"),
            dup_prices.as_os_str(),
            log.as_os_str(),
        ],
        WORK_LIMIT_SECONDS,
    );
    for path in [&prices, &dup_prices, &log] {
        let _ = fs::remove_file(path);
    }
    let output = done.expect("100000 region lines must not take 20 seconds");
    assert_eq!(output.status.code(), Some(0));
    let output = dup_done.expect("a repeated region at the end must not take 20 seconds");
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("dubbele regio r0"), "{err}");
}

/// Time limit for a gzip file of very many members: the promise in the README.
const MANY_MEMBERS_LIMIT_SECONDS: u64 = 5;

/// Wall-clock budget of the member-limit test: it reads four files of 500000
/// members one after the other, which a slow runner (macos-15-intel) cannot do
/// in 5 seconds. The refusal and exit 2 stay asserted; only the time is wider.
const MEMBER_LIMIT_TEST_SECONDS: u64 = 30;

/// The most members a gzip file may hold.
const MAX_GZIP_MEMBERS: usize = 500_000;

const FIELDS_LINE: &str = "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n";

/// A gzip member made with the given header parts and no content.
fn gzip_member_with(builder: flate2::GzBuilder, bytes: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut encoder = builder.write(Vec::new(), flate2::Compression::fast());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

fn many_members_file(label: &str, first: &[u8], member: &[u8], count: usize) -> PathBuf {
    let mut bytes = first.to_vec();
    for _ in 0..count {
        bytes.extend_from_slice(member);
    }
    temp_bytes(label, &bytes)
}

#[test]
fn many_empty_gzip_members_are_read_in_time() {
    let first = gzip_member(FIELDS_LINE.as_bytes());
    let path = many_members_file("many-empty.gz", &first, &gzip_member(b""), 200_000);
    let done = run_within(&[path.as_os_str()], MANY_MEMBERS_LIMIT_SECONDS);
    let _ = fs::remove_file(&path);
    let output = done.expect("200000 empty gzip members must not take 5 seconds");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn many_gzip_members_with_a_bom_each_are_read_in_time() {
    let first = gzip_member(format!("\u{feff}{FIELDS_LINE}").as_bytes());
    let path = many_members_file(
        "many-bom.gz",
        &first,
        &gzip_member("\u{feff}".as_bytes()),
        200_000,
    );
    let done = run_within(&[path.as_os_str()], MANY_MEMBERS_LIMIT_SECONDS);
    let _ = fs::remove_file(&path);
    let output = done.expect("200000 gzip members with a BOM must not take 5 seconds");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn many_gzip_members_with_every_header_part_are_read_in_time() {
    let first = gzip_member(FIELDS_LINE.as_bytes());
    let builder = flate2::GzBuilder::new()
        .filename("a.log")
        .comment("c")
        .extra(vec![1, 2, 3]);
    let member = gzip_member_with(builder, b"");
    let path = many_members_file("many-header-parts.gz", &first, &member, 200_000);
    let done = run_within(&[path.as_os_str()], MANY_MEMBERS_LIMIT_SECONDS);
    let _ = fs::remove_file(&path);
    let output = done.expect("200000 members with a name, comment and extra field must be fast");
    assert_eq!(output.status.code(), Some(0));
}

/// The member count is the work bound: a file of exactly the maximum is read,
/// one member more is refused at once, whatever the members hold.
#[test]
fn a_gzip_file_of_more_members_than_the_maximum_is_refused() {
    let first = gzip_member(FIELDS_LINE.as_bytes());
    let empty = gzip_member(b"");
    // The first member and the empty ones together are the members counted.
    let at_limit = many_members_file("members-at-limit.gz", &first, &empty, MAX_GZIP_MEMBERS - 1);
    let over = many_members_file("members-over.gz", &first, &empty, MAX_GZIP_MEMBERS);
    let bom = gzip_member("\u{feff}".as_bytes());
    let over_bom = many_members_file("members-over-bom.gz", &first, &bom, MAX_GZIP_MEMBERS);
    let named = gzip_member_with(flate2::GzBuilder::new().filename("a"), b"x");
    let over_named = many_members_file("members-over-named.gz", &first, &named, MAX_GZIP_MEMBERS);
    let at_done = run_within(&[at_limit.as_os_str()], MEMBER_LIMIT_TEST_SECONDS);
    let over_done = run_within(&[over.as_os_str()], MEMBER_LIMIT_TEST_SECONDS);
    let bom_done = run_within(&[over_bom.as_os_str()], MEMBER_LIMIT_TEST_SECONDS);
    let named_done = run_within(&[over_named.as_os_str()], MEMBER_LIMIT_TEST_SECONDS);
    for path in [&at_limit, &over, &over_bom, &over_named] {
        let _ = fs::remove_file(path);
    }
    let at = at_done.expect("a file of the maximum number of members must be read in time");
    assert_eq!(at.status.code(), Some(0));
    for (label, done) in [
        ("empty", over_done),
        ("bom", bom_done),
        ("named", named_done),
    ] {
        let output = done.unwrap_or_else(|| panic!("{label}: too many members must end in time"));
        assert_eq!(output.status.code(), Some(2), "{label}");
        let err = String::from_utf8(output.stderr).unwrap();
        assert!(
            err.contains("gzip: more than 500000 members"),
            "{label}: {err}"
        );
    }
}

/// Members with every kind of header must give the same report as one member.
#[test]
fn gzip_members_with_header_parts_give_the_report_of_one_member() {
    let whole = temp_bytes("parts-whole.gz", &gzip_member(SMALL_LOG.as_bytes()));
    let expected = command().arg(&whole).output().unwrap();
    let _ = fs::remove_file(&whole);
    assert_eq!(expected.status.code(), Some(0));
    let bytes = SMALL_LOG.as_bytes();
    let (a, b) = bytes.split_at(20);
    let (b, c) = b.split_at(40);
    let named = flate2::GzBuilder::new().filename("x.log").comment("note");
    let extra = flate2::GzBuilder::new().extra(vec![9, 9]).mtime(7);
    let joined = [
        gzip_member_with(named, a),
        gzip_member_with(extra, b),
        gzip_member(c),
        gzip_member(b""),
    ]
    .concat();
    let path = temp_bytes("parts-joined.gz", &joined);
    let output = command().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, expected.stdout);
}

/// A damaged member in the middle of many good ones is still refused with the
/// same message as a single damaged member.
#[test]
fn a_damaged_member_after_empty_members_is_refused_with_the_same_text() {
    let mut damaged = gzip_member(b"abc");
    let last = damaged.len() - 8;
    damaged[last] ^= 0xff;
    let single = temp_bytes("damaged-single.gz", &damaged);
    let single_err = stderr_of(&[single.as_os_str()]);
    let first = gzip_member(FIELDS_LINE.as_bytes());
    let mut bytes = first.clone();
    for _ in 0..1000 {
        bytes.extend_from_slice(&gzip_member(b""));
    }
    bytes.extend_from_slice(&damaged);
    let many = temp_bytes("damaged-many.gz", &bytes);
    let many_err = stderr_of(&[many.as_os_str()]);
    let _ = fs::remove_file(&single);
    let _ = fs::remove_file(&many);
    let reason = |err: &str| err.split("gzip: ").nth(1).map(str::to_string);
    assert!(reason(&single_err).is_some(), "{single_err}");
    assert_eq!(reason(&single_err), reason(&many_err), "{many_err}");
}

#[test]
fn a_plain_file_with_an_upper_case_gz_name_is_refused_as_gzip() {
    let path = temp_bytes("plain-text.GZ", SMALL_LOG.as_bytes());
    let output = command().arg(&path).output().unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(
        err.contains("plain-text.GZ: gzip: invalid gzip header"),
        "{err}"
    );
}

const ESTIMATE_LINE: &str =
    "Estimate from list prices, not your AWS invoice (free tier, credits, discounts not included)";

#[test]
fn text_labels_the_monthly_cost_as_an_estimate() {
    let output = command().arg(fixture()).output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(text.matches(ESTIMATE_LINE).count(), 1);
    assert!(text.contains("\nestimated_usd_month=0.000000\n"));
    assert!(!text.contains("totaal_usd"));
}

#[test]
fn json_is_marked_as_an_estimate_and_names_the_monthly_cost_alike() {
    let output = command()
        .arg("--format")
        .arg("json")
        .arg(fixture())
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.starts_with("{\n  \"schema_version\": 1,\n  \"estimate\": true,\n"));
    assert!(text.contains("\"estimated_usd_month_micros\": 0,"));
    assert!(!text.contains("total_usd_micros"));
}

#[test]
fn an_empty_report_is_still_an_estimate() {
    let dir = std::env::temp_dir().join(format!("cdn-estimate-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let log = dir.join("empty.log");
    fs::write(&log, "").unwrap();
    for json in [false, true] {
        let mut cmd = command();
        if json {
            cmd.arg("--format").arg("json");
        }
        let text = String::from_utf8(cmd.arg(&log).output().unwrap().stdout).unwrap();
        if json {
            assert!(text.contains("\"estimate\": true"));
        } else {
            assert!(text.contains(ESTIMATE_LINE));
        }
    }
    fs::remove_dir_all(&dir).unwrap();
}

fn old_prices(dir: &std::path::Path) -> PathBuf {
    let text = cdn_egress_bill::embedded_price_text()
        .replace("checked_on 2026-11-09", "checked_on 2026-01-01");
    let path = dir.join("old-prices.txt");
    fs::write(&path, text).unwrap();
    path
}

fn age_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cdn-age-{name}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn prices_older_than_180_days_warn_on_stderr_and_exit_0() {
    let dir = age_dir("old");
    let prices = old_prices(&dir);
    // 2026-01-01 + 180 days = 2026-06-30: one day later is over the limit.
    let output = command()
        .arg("--prices")
        .arg(&prices)
        .arg("--as-of")
        .arg("2026-07-01")
        .arg(fixture())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("2026-01-01"), "{err}");
    assert!(err.contains("180"), "{err}");
    assert_eq!(err.lines().count(), 1);
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("maand: 2026-03"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn prices_of_exactly_180_days_do_not_warn() {
    let dir = age_dir("edge");
    let prices = old_prices(&dir);
    let output = command()
        .arg("--prices")
        .arg(&prices)
        .arg("--as-of")
        .arg("2026-06-30")
        .arg(fixture())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_built_in_prices_warn_when_as_of_is_far_ahead_and_not_when_close() {
    let far = command()
        .arg("--as-of")
        .arg("2027-06-01")
        .arg(fixture())
        .output()
        .unwrap();
    assert_eq!(far.status.code(), Some(0));
    assert!(String::from_utf8(far.stderr)
        .unwrap()
        .contains("2026-11-09"));
    let near = command()
        .arg("--as-of")
        .arg("2026-11-20")
        .arg(fixture())
        .output()
        .unwrap();
    assert!(near.stderr.is_empty());
}

#[test]
fn as_of_must_be_a_real_date() {
    let good = command()
        .arg("--as-of")
        .arg("2026-02-28")
        .arg(fixture())
        .output()
        .unwrap();
    assert_eq!(good.status.code(), Some(0));
    for bad in ["2026-02-30", "yesterday", "2026-1-1", ""] {
        let output = command()
            .arg("--as-of")
            .arg(bad)
            .arg(fixture())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{bad}");
    }
}

fn run_with(args: &[&str]) -> (Option<i32>, String, String) {
    let output = command().args(args).arg(fixture()).output().unwrap();
    (
        output.status.code(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

#[test]
fn quiet_keeps_the_report_and_drops_the_price_age_warning() {
    let dir = age_dir("quiet");
    let prices = old_prices(&dir);
    let prices = prices.to_str().unwrap();
    let loud = run_with(&["--prices", prices, "--as-of", "2026-11-20"]);
    assert_eq!(loud.0, Some(0));
    assert!(loud.2.contains("2026-01-01"), "{}", loud.2);
    let quiet = run_with(&["--quiet", "--prices", prices, "--as-of", "2026-11-20"]);
    assert_eq!(quiet.0, Some(0));
    assert!(quiet.2.is_empty(), "{}", quiet.2);
    assert_eq!(quiet.1, loud.1);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn quiet_still_reports_errors_and_the_strict_exit() {
    let strict = run_with(&["--quiet", "--strict", "--as-of", "2026-11-20"]);
    assert_eq!(strict.0, Some(3));
    assert!(strict.1.contains("onbekend"));
    let missing = command()
        .arg("--quiet")
        .arg("/nonexistent/cdn.log")
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(2));
    assert!(missing.stdout.is_empty());
    assert!(!missing.stderr.is_empty());
}

#[test]
fn verbose_adds_progress_on_stderr_and_leaves_stdout_alone() {
    let plain = run_with(&["--as-of", "2026-11-20"]);
    let verbose = run_with(&["--verbose", "--as-of", "2026-11-20"]);
    assert_eq!(verbose.0, Some(0));
    assert_eq!(verbose.1, plain.1);
    assert!(plain.2.is_empty());
    assert!(verbose.2.lines().count() >= 3, "{}", verbose.2);
    assert!(verbose.2.contains("twee_regios.log"), "{}", verbose.2);
    assert!(verbose.2.contains("4 records"), "{}", verbose.2);
}

#[test]
fn quiet_and_verbose_together_are_a_usage_error() {
    for args in [["--quiet", "--verbose"], ["--verbose", "--quiet"]] {
        let (code, out, err) = run_with(&args);
        assert_eq!(code, Some(2), "{err}");
        assert!(out.is_empty());
        assert!(err.contains("cannot be used with"), "{err}");
    }
}

fn md_run(args: &[&str], log: &std::path::Path) -> String {
    let output = command()
        .args(args)
        .arg("--as-of")
        .arg("2026-11-20")
        .arg(log)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn markdown_has_the_same_amounts_as_text_and_json() {
    let log = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/documented_full.log");
    let md = md_run(&["--md"], &log);
    let text = md_run(&[], &log);
    let json = md_run(&["--format", "json"], &log);
    assert!(md.contains(ESTIMATE_LINE));
    assert_eq!(md.matches(ESTIMATE_LINE).count(), 1);
    assert!(md.contains("\nestimated_usd_month=0.085000\n"));
    assert!(json.contains("\"estimated_usd_month_micros\": 85000,"));
    assert!(md.contains("| US/Mexico/Canada | IAD | 1100585369600 | 1 | 2 | 0 | 0.085000 | ja |"));
    // Every tab separated row of the text report is a row of the table with the same cells.
    for line in text.lines().filter(|line| line.contains('\t')) {
        let cells: Vec<&str> = line.split('\t').collect();
        let row = format!("| {} |", cells.join(" | "));
        assert!(md.contains(&row), "{row}\n{md}");
    }
    for line in text.lines().filter(|line| line.contains(": ")) {
        if line.starts_with("onberekend_") || line.starts_with("gratis_") {
            assert!(md.contains(&format!("{line}\n")), "{line}");
        }
    }
    assert!(md.contains("\n## 2026-10\n"));
}

#[test]
fn markdown_escapes_pipes_and_backslashes_in_cells() {
    let dir = age_dir("md");
    let log = dir.join("pipe.log");
    fs::write(
        &log,
        "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n2026-03-01\tIAD1\t5\t/a|b\\c\thttps\n",
    )
    .unwrap();
    let md = md_run(&["--md"], &log);
    assert!(
        md.contains("| /a\\|b\\\\c | 5 | 1 | 0.000000 | 0 | 0 |"),
        "{md}"
    );
    for line in md.lines().filter(|line| line.starts_with("| /a")) {
        assert_eq!(
            line.replace("\\\\", "")
                .replace("\\|", "")
                .matches('|')
                .count(),
            7
        );
    }
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn markdown_for_an_empty_log_and_the_conflict_with_format() {
    let dir = age_dir("mdempty");
    let log = dir.join("empty.log");
    fs::write(&log, "").unwrap();
    let md = md_run(&["--md"], &log);
    assert!(md.contains(ESTIMATE_LINE));
    assert!(md.contains("geen verkeer"));
    let both = command()
        .args(["--md", "--format", "json"])
        .arg(&log)
        .output()
        .unwrap();
    assert_eq!(both.status.code(), Some(2));
    assert!(both.stdout.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}
