use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use flate2::bufread::GzDecoder;

use crate::error::Error;
use crate::money::is_date;

/// The most members a gzip log may hold. A decoder is set up for every member,
/// about 4 microseconds each, so this keeps a file under the size limit well
/// below 5 seconds whatever its members hold.
const MAX_GZIP_MEMBERS: usize = 500_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub month: String,
    pub location: String,
    pub bytes: u64,
    pub path: String,
    pub protocol: String,
}

struct Columns {
    date: usize,
    location: usize,
    bytes: usize,
    path: usize,
    protocol: usize,
    width: usize,
}

/// True when the stat is a fifo that is this process's own stdin (a pipe we may read).
#[cfg(unix)]
fn is_own_stdin_pipe(before: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::{FileTypeExt, MetadataExt};
    if !before.file_type().is_fifo() {
        return false;
    }
    std::fs::metadata("/dev/fd/0")
        .is_ok_and(|stdin| stdin.dev() == before.dev() && stdin.ino() == before.ino())
}

#[cfg(not(unix))]
fn is_own_stdin_pipe(_before: &std::fs::Metadata) -> bool {
    false
}

/// Bovengrens voor een gewoon bestand (logs, prijzen, edges).
pub const MAX_FILE_BYTES: u64 = 268_435_456;

/// Path as shown in errors: valid UTF-8 equals `path.display()` unless it holds control or bidi characters (then `visible`); otherwise the raw
/// bytes as `escape_ascii`, never U+FFFD.
pub fn path_text(path: &Path) -> String {
    match path.to_str() {
        Some(text) if text.chars().any(crate::report::needs_escape) => crate::report::visible(text),
        Some(text) => text.to_owned(),
        None => raw_bytes(path).escape_ascii().to_string(),
    }
}

#[cfg(unix)]
fn raw_bytes(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    path.as_os_str().as_bytes().to_vec()
}

#[cfg(not(unix))]
fn raw_bytes(path: &Path) -> Vec<u8> {
    path.to_string_lossy().into_owned().into_bytes()
}

/// Weigert een te groot gewoon bestand vóór het lezen.
pub fn check_file_size(path: &Path, len: u64) -> Result<(), Error> {
    if len > MAX_FILE_BYTES {
        return Err(Error::new(format!(
            "{}: file is larger than {MAX_FILE_BYTES} bytes",
            path_text(path)
        )));
    }
    Ok(())
}

pub fn read_log_file(path: &Path) -> Result<String, Error> {
    // Eerst stat: een fifo openen blokkeert zolang er geen schrijver is.
    let before =
        std::fs::metadata(path).map_err(|err| Error::new(format!("{}: {err}", path_text(path))))?;
    let io_err = |err: std::io::Error| Error::new(format!("{}: {err}", path_text(path)));
    let mut data = Vec::new();
    if before.is_file() {
        check_file_size(path, before.len())?;
        File::open(path)
            .and_then(|mut file| file.read_to_end(&mut data))
            .map_err(io_err)?;
    } else if is_own_stdin_pipe(&before) {
        // Onze eigen stdin: lezen zonder openen of seek, dus geen hang en geen "Illegal seek".
        // Een pipe heeft geen lengte: tel wat binnenkomt, in stukken, en stop boven de grens.
        let mut stdin = std::io::stdin().lock();
        let mut chunk = vec![0u8; 64 * 1024];
        loop {
            let n = match stdin.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => n,
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(err) => return Err(io_err(err)),
            };
            if (data.len() as u64).saturating_add(n as u64) > MAX_FILE_BYTES {
                check_file_size(path, MAX_FILE_BYTES + 1)?;
            }
            data.extend_from_slice(&chunk[..n]);
        }
    } else {
        return Err(Error::new(format!(
            "{}: geen gewoon bestand",
            path_text(path)
        )));
    }
    let magic = data.get(..2).is_some_and(|head| head == [0x1f, 0x8b]);
    let named_gz = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"));
    let gzip = named_gz || magic;
    let text = if gzip {
        let fail = |err: std::io::Error| Error::new(format!("{}: gzip: {err}", path_text(path)));
        let mut rest: &[u8] = &data;
        let mut first = true;
        let mut members = 0usize;
        let mut joined: Vec<u8> = Vec::new();
        // Decode member by member: each member may start with its own BOM, and
        // zero padding after a member is accepted like gzip(1) does. The members
        // are joined as bytes: a UTF-8 character may be split over two members.
        while first || !(rest.is_empty() || rest.iter().all(|&byte| byte == 0)) {
            if !first && !rest.starts_with(&[0x1f, 0x8b]) {
                return Err(Error::new(format!(
                    "{}: gzip: trailing data is not a gzip member",
                    path_text(path)
                )));
            }
            first = false;
            members += 1;
            if members > MAX_GZIP_MEMBERS {
                return Err(Error::new(format!(
                    "{}: gzip: more than {MAX_GZIP_MEMBERS} members",
                    path_text(path)
                )));
            }
            let start = joined.len();
            let mut decoder = GzDecoder::new(&mut rest);
            if let Err(err) = decoder.read_to_end(&mut joined) {
                // The library reports a cut-off header and a cut-off body with the
                // same error; header() is None until the header is complete.
                if err.kind() == std::io::ErrorKind::UnexpectedEof && decoder.header().is_none() {
                    return Err(Error::new(format!(
                        "{}: gzip: truncated gzip header",
                        path_text(path)
                    )));
                }
                return Err(fail(err));
            }
            if joined[start..].starts_with(&[0xef, 0xbb, 0xbf]) {
                joined.drain(start..start + 3);
            }
        }
        String::from_utf8(joined).map_err(|_| {
            fail(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "stream did not contain valid UTF-8",
            ))
        })?
    } else {
        String::from_utf8(data).map_err(|_| {
            io_err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "stream did not contain valid UTF-8",
            ))
        })?
    };
    Ok(text)
}

pub fn load_logs(paths: &[impl AsRef<Path>]) -> Result<Vec<Record>, Error> {
    let mut records = Vec::new();
    for path in paths {
        let path = path.as_ref();
        let text = read_log_file(path)?;
        records.extend(parse_log(&path_text(path), &text)?);
    }
    Ok(records)
}

/// Split on LF, CRLF and a lone CR, so a log whose lines end only in CR still counts.
fn split_lines(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                lines.push(&text[start..i]);
                start = i + 1;
            }
            b'\r' => {
                lines.push(&text[start..i]);
                if bytes.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

pub fn parse_log(name: &str, text: &str) -> Result<Vec<Record>, Error> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut columns: Option<Columns> = None;
    let mut records = Vec::new();
    for (lineno, raw) in split_lines(text).into_iter().enumerate() {
        let line_no = lineno + 1;
        let line = raw.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("#Fields:") {
            columns = Some(columns_from(name, line_no, rest)?);
            continue;
        }
        if line.starts_with('#') && !line.contains('\t') {
            continue;
        }
        let Some(columns) = columns.as_ref() else {
            return Err(Error::new(format!(
                "{name}:{line_no}: geen #Fields vóór de data"
            )));
        };
        let mut fields: Vec<&str> = line.split('\t').collect();
        if fields.last().is_some_and(|field| field.is_empty()) {
            fields.pop();
        }
        if fields.len() != columns.width {
            return Err(Error::new(format!(
                "{name}:{line_no}: {0} velden, #Fields heeft {1}",
                fields.len(),
                columns.width
            )));
        }
        let date = fields[columns.date];
        if !is_date(date) {
            return Err(Error::new(format!(
                "{name}:{line_no}: datum {date} is ongeldig"
            )));
        }
        let bytes = parse_u64(fields[columns.bytes])
            .ok_or_else(|| Error::new(format!("{name}:{line_no}: sc-bytes is geen getal")))?;
        records.push(Record {
            month: date[..7].to_string(),
            location: fields[columns.location].to_string(),
            bytes,
            path: fields[columns.path].to_string(),
            protocol: fields[columns.protocol].to_ascii_lowercase(),
        });
    }
    Ok(records)
}

fn columns_from(name: &str, line_no: usize, rest: &str) -> Result<Columns, Error> {
    if rest.contains('\u{a0}') {
        return Err(Error::new(format!(
            "{name}:{line_no}: #Fields bevat een NBSP"
        )));
    }
    let fields: Vec<&str> = rest
        .split([' ', '\t', '\r', '\n'])
        .filter(|field| !field.is_empty())
        .collect();
    let mut seen = HashSet::with_capacity(fields.len());
    for field in &fields {
        if !seen.insert(*field) {
            return Err(Error::new(format!(
                "{name}:{line_no}: #Fields dubbele naam {field}"
            )));
        }
    }
    let pos = |field: &str| {
        fields
            .iter()
            .position(|name| *name == field)
            .ok_or_else(|| Error::new(format!("{name}:{line_no}: #Fields mist {field}")))
    };
    Ok(Columns {
        date: pos("date")?,
        location: pos("x-edge-location")?,
        bytes: pos("sc-bytes")?,
        path: pos("cs-uri-stem")?,
        protocol: pos("cs-protocol")?,
        width: fields.len(),
    })
}

fn parse_u64(token: &str) -> Option<u64> {
    if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    token.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    fn sample() -> &'static str {
        "#Version: 1.0\n#Fields: date time x-edge-location sc-bytes c-ip cs-method cs(Host) cs-uri-stem sc-status cs-protocol\n2026-03-01\t12:00:00\tAMS50-C1\t1000\t203.0.113.10\tGET\texample.net\t/video.mp4\t200\thttps\n"
    }

    #[test]
    fn parses_tab_separated_standard_log() {
        let records = parse_log("t.log", sample()).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].month, "2026-03");
        assert_eq!(records[0].location, "AMS50-C1");
        assert_eq!(records[0].bytes, 1000);
        assert_eq!(records[0].path, "/video.mp4");
        assert_eq!(records[0].protocol, "https");
    }

    #[test]
    fn field_order_does_not_matter() {
        let text = "#Fields: cs-protocol cs-uri-stem sc-bytes x-edge-location date\nhttps\t/a\t10\tAMS1\t2026-01-02\n";
        let records = parse_log("t", text).unwrap();
        assert_eq!(records[0].path, "/a");
        assert_eq!(records[0].bytes, 10);
        assert_eq!(records[0].protocol, "https");
        assert_eq!(records[0].location, "AMS1");
    }

    #[test]
    fn cost_basis_is_sc_bytes_not_sc_content_len() {
        let text = "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol sc-content-len\n2026-01-02\tIAD1\t392\t/a\thttp\t78\n";
        let records = parse_log("t", text).unwrap();
        assert_eq!(records[0].bytes, 392);
    }

    #[test]
    fn missing_required_field_is_an_error() {
        let text =
            "#Fields: date x-edge-location cs-uri-stem cs-protocol\n2026-01-02\tIAD1\t/a\thttp\n";
        let err = parse_log("t", text).unwrap_err();
        assert!(err.message.contains("sc-bytes"));
    }

    #[test]
    fn data_before_fields_is_an_error() {
        let err = parse_log("t", "2026-01-02\tIAD1\t1\t/a\thttp\n").unwrap_err();
        assert!(err.message.contains("#Fields"));
    }

    #[test]
    fn bad_sc_bytes_names_the_line() {
        let text = "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n2026-01-02\tIAD1\t-\t/a\thttp\n";
        let err = parse_log("demo.log", text).unwrap_err();
        assert!(err.message.contains("demo.log:2"));
        assert!(err.message.contains("sc-bytes"));
    }

    #[test]
    fn crlf_comments_and_blank_lines() {
        let text = "#Version: 1.0\r\n\r\n#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\r\n2026-04-05\tFRA1\t7\t/b\tHTTP\r\n";
        let records = parse_log("t", text).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].protocol, "http");
        assert_eq!(records[0].month, "2026-04");
    }

    #[test]
    fn hash_first_field_row_counts_the_bytes() {
        let text = "#Fields: cs(Referer) date x-edge-location sc-bytes cs-uri-stem cs-protocol\n#frag\t2026-01-01\tAMS1\t900\t/b\thttps\n";
        let records = parse_log("t", text).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].bytes, 900);
    }

    #[test]
    fn cr_only_lines_count_the_bytes() {
        let text = "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\r2026-01-01\tAMS1\t1\t/a\thttps\r";
        let records = parse_log("t", text).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].bytes, 1);
    }

    #[test]
    fn gzip_log_is_read() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("cdn-egress-bill-{}-gz.log.gz", std::process::id()));
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(sample().as_bytes()).unwrap();
        std::fs::write(&path, encoder.finish().unwrap()).unwrap();
        let records = load_logs(std::slice::from_ref(&path)).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(records[0].path, "/video.mp4");
        assert_eq!(records[0].bytes, 1000);
    }

    #[test]
    fn gzip_with_two_members_counts_both() {
        let first = "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n2026-01-02\tAMS1\t11\t/a\thttps\n";
        let second = "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n2026-01-03\tIAD1\t22\t/b\thttps\n";
        let mut bytes = Vec::new();
        for text in [first, second] {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(text.as_bytes()).unwrap();
            bytes.extend(encoder.finish().unwrap());
        }
        assert!(bytes.len() < 4096);
        let path =
            std::env::temp_dir().join(format!("cdn-egress-bill-{}-multi.gz", std::process::id()));
        std::fs::write(&path, &bytes).unwrap();
        let records = load_logs(std::slice::from_ref(&path)).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].path, "/a");
        assert_eq!(records[0].bytes, 11);
        assert_eq!(records[1].path, "/b");
        assert_eq!(records[1].bytes, 22);
    }

    #[test]
    fn gzip_second_member_with_bom_and_zero_padding() {
        let head = "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n";
        let first = format!("{head}2026-01-02\tAMS1\t11\t/a\thttps\n");
        let second = format!("\u{feff}{head}2026-01-03\tIAD1\t22\t/b\thttps\n");
        let mut bytes = Vec::new();
        for text in [&first, &second] {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(text.as_bytes()).unwrap();
            bytes.extend(encoder.finish().unwrap());
        }
        let path =
            std::env::temp_dir().join(format!("cdn-egress-bill-{}-bom2.gz", std::process::id()));
        std::fs::write(&path, &bytes).unwrap();
        let records = load_logs(std::slice::from_ref(&path)).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[1].bytes, 22);
        bytes.extend([0u8; 4]);
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(load_logs(std::slice::from_ref(&path)).unwrap().len(), 2);
        bytes.extend(b"junk");
        std::fs::write(&path, &bytes).unwrap();
        let junk = load_logs(std::slice::from_ref(&path));
        let _ = std::fs::remove_file(&path);
        let message = junk.unwrap_err().to_string();
        assert!(message.contains("trailing data is not a gzip member"));
        assert!(!message.contains("unexpected end of file"));
        assert!(message.contains(&path.display().to_string()));
        assert!(!message.contains("junk"));
    }

    #[test]
    fn gzip_truncated_header_differs_from_truncated_body() {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(b"#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n")
            .unwrap();
        let member = encoder.finish().unwrap();
        let path =
            std::env::temp_dir().join(format!("cdn-egress-bill-{}-trunc.gz", std::process::id()));
        let mut header_cut = member.clone();
        header_cut.extend([0x1f, 0x8b, 0x08]);
        std::fs::write(&path, &header_cut).unwrap();
        let header_msg = load_logs(std::slice::from_ref(&path))
            .unwrap_err()
            .to_string();
        let body_cut = &member[..member.len() - 4];
        std::fs::write(&path, body_cut).unwrap();
        let body_msg = load_logs(std::slice::from_ref(&path))
            .unwrap_err()
            .to_string();
        let _ = std::fs::remove_file(&path);
        assert!(header_msg.contains("truncated gzip header"), "{header_msg}");
        assert!(body_msg.contains("unexpected end of file"), "{body_msg}");
        assert_ne!(header_msg, body_msg);
    }

    #[test]
    fn gzip_member_then_junk_without_padding() {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(b"#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n")
            .unwrap();
        let mut bytes = encoder.finish().unwrap();
        bytes.extend(b"junk");
        let path =
            std::env::temp_dir().join(format!("cdn-egress-bill-{}-junk.gz", std::process::id()));
        std::fs::write(&path, &bytes).unwrap();
        let result = load_logs(std::slice::from_ref(&path));
        let _ = std::fs::remove_file(&path);
        let message = result.unwrap_err().to_string();
        assert!(message.contains("trailing data is not a gzip member"));
        assert!(!message.contains("unexpected end of file"));
        assert!(message.contains(&path.display().to_string()));
    }

    #[test]
    fn bom_is_stripped() {
        let text = format!("\u{feff}{0}", sample());
        let records = parse_log("t", &text).unwrap();
        assert_eq!(records[0].location, "AMS50-C1");
    }

    #[test]
    fn http_status_404_still_counts() {
        let text = "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol sc-status\n2026-01-02\tIAD1\t40\t/missing\thttps\t404\n";
        let records = parse_log("t", text).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].bytes, 40);
        assert_eq!(records[0].path, "/missing");
    }

    #[test]
    fn uri_stem_is_the_path() {
        let text = "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-uri-query cs-protocol\n2026-01-02\tIAD1\t8\t/a%20b\tx=1\thttps\n";
        let records = parse_log("t", text).unwrap();
        assert_eq!(records[0].path, "/a%20b");
    }

    #[test]
    fn repeated_field_name_is_an_error() {
        let repeated = "\
#Version: 1.0
#Fields: date x-edge-location sc-bytes sc-bytes cs-uri-stem cs-protocol
2026-03-01\tAMS1\t10\t99\t/a\thttps
";
        let err = parse_log("dup.log", repeated).unwrap_err();
        assert!(err.message.contains("dup.log:2"), "{}", err.message);
        assert!(err.message.contains("dubbele"), "{}", err.message);
        assert!(err.message.contains("sc-bytes"), "{}", err.message);

        let optional = "\
#Fields: date time time x-edge-location sc-bytes cs-uri-stem cs-protocol
2026-03-01\t12:00:00\t12:00:00\tAMS1\t10\t/a\thttps
";
        let err = parse_log("dup.log", optional).unwrap_err();
        assert!(err.message.contains("dubbele"), "{}", err.message);
        assert!(err.message.contains("time"), "{}", err.message);

        let distinct = "\
#Fields: date time x-edge-location sc-bytes cs-uri-stem cs-protocol
2026-03-01\t12:00:00\tAMS1\t10\t/a\thttps
";
        let records = parse_log("ok.log", distinct).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].bytes, 10);
        assert_eq!(records[0].path, "/a");
        assert_eq!(records[0].location, "AMS1");
    }

    #[test]
    fn nbsp_in_fields_line_is_an_error() {
        let text = "#Fields: date x-edge-location sc-bytes cs-uri-stem\u{a0}cs-protocol\n";
        assert_eq!(
            log_message("n.log", text),
            "n.log:1: #Fields bevat een NBSP"
        );
    }

    fn log_message(name: &str, text: &str) -> String {
        parse_log(name, text).unwrap_err().message
    }

    #[test]
    fn log_error_texts_are_fixed() {
        assert_eq!(
            log_message("t", "2026-01-02\tIAD1\t1\t/a\thttp\n"),
            "t:1: geen #Fields vóór de data"
        );
        let head = "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n";
        assert_eq!(
            log_message("t", &format!("{head}2026-01-02\tIAD1\t1\t/a\n")),
            "t:2: 4 velden, #Fields heeft 5"
        );
        assert_eq!(
            log_message("t", &format!("{head}2026-13-01\tIAD1\t1\t/a\thttp\n")),
            "t:2: datum 2026-13-01 is ongeldig"
        );
        assert_eq!(
            log_message(
                "demo.log",
                &format!("{head}2026-01-02\tIAD1\t-\t/a\thttp\n")
            ),
            "demo.log:2: sc-bytes is geen getal"
        );
        assert_eq!(
            log_message(
                "dup.log",
                "#Version: 1.0\n#Fields: date x-edge-location sc-bytes sc-bytes cs-uri-stem cs-protocol\n"
            ),
            "dup.log:2: #Fields dubbele naam sc-bytes"
        );
    }

    #[test]
    fn missing_field_error_text_names_each_required_field() {
        let all = [
            "date",
            "x-edge-location",
            "sc-bytes",
            "cs-uri-stem",
            "cs-protocol",
        ];
        for skip in all {
            let kept: Vec<&str> = all.iter().copied().filter(|name| *name != skip).collect();
            let text = format!("#Fields: {}\n", kept.join(" "));
            assert_eq!(log_message("t", &text), format!("t:1: #Fields mist {skip}"));
        }
    }

    #[test]
    fn broken_gzip_error_text_is_fixed() {
        let path =
            std::env::temp_dir().join(format!("cdn-egress-bill-{}-broken.gz", std::process::id()));
        std::fs::write(&path, [0x1f, 0x8b, 0x08, 0x00, 0xff, 0xff]).unwrap();
        let result = load_logs(std::slice::from_ref(&path));
        let _ = std::fs::remove_file(&path);
        let message = result.unwrap_err().message;
        assert!(
            message.starts_with(&format!("{}: gzip: ", path.display())),
            "{message}"
        );
    }

    #[test]
    fn two_files_keep_their_own_headers() {
        let dir = std::env::temp_dir();
        let first = dir.join(format!("cdn-egress-bill-{}-a.log", std::process::id()));
        let second = dir.join(format!("cdn-egress-bill-{}-b.log", std::process::id()));
        std::fs::write(
            &first,
            "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n2026-01-02\tAMS1\t1\t/a\thttps\n",
        )
        .unwrap();
        std::fs::write(
            &second,
            "#Fields: cs-protocol cs-uri-stem sc-bytes x-edge-location date\nhttp\t/b\t2\tIAD1\t2026-01-03\n",
        )
        .unwrap();
        let records = load_logs(&[first.clone(), second.clone()]).unwrap();
        let _ = std::fs::remove_file(&first);
        let _ = std::fs::remove_file(&second);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].path, "/a");
        assert_eq!(records[1].path, "/b");
        assert_eq!(records[1].bytes, 2);
    }
}
