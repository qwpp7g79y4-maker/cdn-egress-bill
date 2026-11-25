use crate::bill::Report;
use crate::money::format_usd;

pub const NOTE: &str =
    "regio = prijsregio van de edge-locatie (x-edge-location), niet het land van de kijker";
pub const ESTIMATE: &str =
    "Estimate from list prices, not your AWS invoice (free tier, credits, discounts not included)";
pub const EXCLUDES: &str = "geen flat-rate, geen korting, geen Savings Bundle";

pub fn render_text(report: &Report) -> String {
    let mut out = String::new();
    out.push_str(&format!("bron: {}\n", visible(&report.source_url)));
    out.push_str(&format!("gecontroleerd: {}\n", visible(&report.checked_on)));
    out.push_str(&format!("bytes_per_gb: {}\n", report.bytes_per_gb));
    out.push_str(&format!("gratis_data_bytes: {}\n", report.free_data_bytes));
    out.push_str(&format!("gratis_verzoeken: {}\n", report.free_requests));
    out.push_str(ESTIMATE);
    out.push('\n');
    out.push_str(EXCLUDES);
    out.push('\n');
    out.push_str(NOTE);
    out.push_str("\n\n");
    if report.months.is_empty() {
        out.push_str("geen verkeer\n");
        return out;
    }
    for (index, month) in report.months.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&format!("maand: {}\n", month.month));
        out.push_str("per regio\n");
        out.push_str("regio\tedges\tbytes\thttp\thttps\toverig\tusd\tberekend\n");
        for region in &month.regions {
            let usd = if region.priced {
                format_usd(region.usd_micros)
            } else {
                "-".to_string()
            };
            let priced = if region.priced { "ja" } else { "nee" };
            let edges = visible_edges(&region.edges);
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                visible(&region.label),
                edges,
                region.bytes,
                region.http_requests,
                region.https_requests,
                region.other_requests,
                usd,
                priced
            ));
        }
        out.push_str("per pad\n");
        out.push_str("pad\tbytes\tverzoeken\tusd\tonberekend_bytes\tonberekend_verzoeken\n");
        for path in &month.paths {
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\n",
                visible(&path.path),
                path.bytes,
                path.requests,
                format_usd(path.usd_micros),
                path.unpriced_bytes,
                path.unpriced_requests
            ));
        }
        out.push_str(&format!(
            "estimated_usd_month={}\n",
            format_usd(month.total_usd_micros)
        ));
        out.push_str(&format!(
            "gratis_data_toegepast: {}\n",
            month.free_data_bytes_applied
        ));
        out.push_str(&format!(
            "gratis_verzoeken_toegepast: {}\n",
            month.free_requests_applied
        ));
        out.push_str(&format!("onberekend_bytes: {}\n", month.unpriced_bytes));
        out.push_str(&format!(
            "onberekend_verzoeken: {}\n",
            month.unpriced_requests
        ));
    }
    out
}

/// A table cell: control characters shown as `\u00xx`, then `\` and `|` escaped.
fn md_cell(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in visible(value).chars() {
        if ch == '\\' || ch == '|' {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

fn md_row(out: &mut String, cells: &[String]) {
    out.push_str("| ");
    out.push_str(&cells.join(" | "));
    out.push_str(" |\n");
}

fn md_header(out: &mut String, names: &[&str]) {
    let cells: Vec<String> = names.iter().map(|name| (*name).to_string()).collect();
    md_row(out, &cells);
    let rule: Vec<String> = names.iter().map(|_| "---".to_string()).collect();
    md_row(out, &rule);
}

/// The report as Markdown: the same amounts as the text report, in tables.
pub fn render_markdown(report: &Report) -> String {
    let mut out = String::from("# CDN egress estimate\n\n");
    out.push_str(ESTIMATE);
    out.push_str("\n\n");
    out.push_str(&format!("- bron: {}\n", visible(&report.source_url)));
    out.push_str(&format!(
        "- gecontroleerd: {}\n",
        visible(&report.checked_on)
    ));
    out.push_str(&format!("- bytes_per_gb: {}\n", report.bytes_per_gb));
    out.push_str(&format!(
        "- gratis_data_bytes: {}\n",
        report.free_data_bytes
    ));
    out.push_str(&format!("- gratis_verzoeken: {}\n\n", report.free_requests));
    out.push_str(EXCLUDES);
    out.push_str("\n\n");
    out.push_str(NOTE);
    out.push('\n');
    if report.months.is_empty() {
        out.push_str("\ngeen verkeer\n");
        return out;
    }
    for month in &report.months {
        out.push_str(&format!("\n## {}\n\n### per regio\n\n", month.month));
        md_header(
            &mut out,
            &[
                "regio", "edges", "bytes", "http", "https", "overig", "usd", "berekend",
            ],
        );
        for region in &month.regions {
            let usd = if region.priced {
                format_usd(region.usd_micros)
            } else {
                "-".to_string()
            };
            md_row(
                &mut out,
                &[
                    md_cell(&region.label),
                    md_cell(&region.edges.join(",")),
                    region.bytes.to_string(),
                    region.http_requests.to_string(),
                    region.https_requests.to_string(),
                    region.other_requests.to_string(),
                    usd,
                    if region.priced { "ja" } else { "nee" }.to_string(),
                ],
            );
        }
        out.push_str("\n### per pad\n\n");
        md_header(
            &mut out,
            &[
                "pad",
                "bytes",
                "verzoeken",
                "usd",
                "onberekend_bytes",
                "onberekend_verzoeken",
            ],
        );
        for path in &month.paths {
            md_row(
                &mut out,
                &[
                    md_cell(&path.path),
                    path.bytes.to_string(),
                    path.requests.to_string(),
                    format_usd(path.usd_micros),
                    path.unpriced_bytes.to_string(),
                    path.unpriced_requests.to_string(),
                ],
            );
        }
        out.push_str(&format!(
            "\nestimated_usd_month={}\n\n",
            format_usd(month.total_usd_micros)
        ));
        out.push_str(&format!(
            "- gratis_data_toegepast: {}\n- gratis_verzoeken_toegepast: {}\n- onberekend_bytes: {}\n- onberekend_verzoeken: {}\n",
            month.free_data_bytes_applied,
            month.free_requests_applied,
            month.unpriced_bytes,
            month.unpriced_requests
        ));
    }
    out
}

pub fn render_json(report: &Report) -> String {
    let mut out = String::from("{\n  \"schema_version\": 1,\n  \"estimate\": true,\n");
    json_str(&mut out, 1, "source_url", &report.source_url, true);
    json_str(&mut out, 1, "checked_on", &report.checked_on, true);
    json_num(&mut out, 1, "bytes_per_gb", report.bytes_per_gb, true);
    json_num(&mut out, 1, "free_data_bytes", report.free_data_bytes, true);
    json_num(&mut out, 1, "free_requests", report.free_requests, true);
    json_str(&mut out, 1, "note", NOTE, true);
    json_str(&mut out, 1, "excludes", EXCLUDES, true);
    indent(&mut out, 1);
    out.push_str("\"months\": [");
    if report.months.is_empty() {
        out.push_str("]\n}\n");
        return out;
    }
    out.push('\n');
    for (index, month) in report.months.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        indent(&mut out, 2);
        out.push_str("{\n");
        json_str(&mut out, 3, "month", &month.month, true);
        indent(&mut out, 3);
        out.push_str("\"regions\": [\n");
        for (region_index, region) in month.regions.iter().enumerate() {
            if region_index > 0 {
                out.push_str(",\n");
            }
            indent(&mut out, 4);
            out.push_str("{\n");
            json_str(&mut out, 5, "id", &region.id, true);
            json_str(&mut out, 5, "label", &region.label, true);
            json_string_array(&mut out, 5, "edges", &region.edges, true);
            json_bool(&mut out, 5, "priced", region.priced, true);
            json_num(&mut out, 5, "bytes", region.bytes, true);
            json_num(&mut out, 5, "http_requests", region.http_requests, true);
            json_num(&mut out, 5, "https_requests", region.https_requests, true);
            json_num(&mut out, 5, "other_requests", region.other_requests, true);
            json_num(&mut out, 5, "usd_micros", region.usd_micros, false);
            indent(&mut out, 4);
            out.push('}');
        }
        if !month.regions.is_empty() {
            out.push('\n');
        }
        indent(&mut out, 3);
        out.push_str("],\n");
        indent(&mut out, 3);
        out.push_str("\"paths\": [\n");
        for (path_index, path) in month.paths.iter().enumerate() {
            if path_index > 0 {
                out.push_str(",\n");
            }
            indent(&mut out, 4);
            out.push_str("{\n");
            json_str(&mut out, 5, "path", &path.path, true);
            json_num(&mut out, 5, "bytes", path.bytes, true);
            json_num(&mut out, 5, "requests", path.requests, true);
            json_num(&mut out, 5, "usd_micros", path.usd_micros, true);
            json_num(&mut out, 5, "unpriced_bytes", path.unpriced_bytes, true);
            json_num(
                &mut out,
                5,
                "unpriced_requests",
                path.unpriced_requests,
                false,
            );
            indent(&mut out, 4);
            out.push('}');
        }
        if !month.paths.is_empty() {
            out.push('\n');
        }
        indent(&mut out, 3);
        out.push_str("],\n");
        json_string_array(&mut out, 3, "unknown_codes", &month.unknown_codes, true);
        json_num(
            &mut out,
            3,
            "estimated_usd_month_micros",
            month.total_usd_micros,
            true,
        );
        json_num(
            &mut out,
            3,
            "free_data_bytes_applied",
            month.free_data_bytes_applied,
            true,
        );
        json_num(
            &mut out,
            3,
            "free_requests_applied",
            month.free_requests_applied,
            true,
        );
        json_num(&mut out, 3, "unpriced_bytes", month.unpriced_bytes, true);
        json_num(
            &mut out,
            3,
            "unpriced_requests",
            month.unpriced_requests,
            false,
        );
        indent(&mut out, 2);
        out.push('}');
    }
    out.push_str("\n  ]\n}\n");
    out
}

/// Stuurtekens als `\u00xx`, zodat een pad of edge de terminal niet aanstuurt.
fn push_visible(out: &mut String, value: &str) {
    for ch in value.chars() {
        if needs_escape(ch) {
            let code = u32::from(ch);
            if code > 0xffff {
                out.push_str(&format!("\\u{{{code:x}}}"));
            } else {
                out.push_str(&format!("\\u{code:04x}"));
            }
        } else {
            out.push(ch);
        }
    }
}

/// Stuurtekens en onzichtbare bidi- of regelscheidingstekens.
pub(crate) fn needs_escape(ch: char) -> bool {
    ch.is_control()
        || matches!(
            ch,
            '\u{200b}'..='\u{200f}'
                | '\u{2028}'..='\u{202e}'
                | '\u{2060}'..='\u{2069}'
                | '\u{061c}'
                | '\u{feff}'
                | '\u{00ad}'
                | '\u{034f}'
                | '\u{2800}'
                | '\u{3164}'
                | '\u{180e}'
                | '\u{206a}'..='\u{206f}'
                | '\u{fe0f}'
                | '\u{115f}'
                | '\u{1160}'
                | '\u{17b4}'
                | '\u{17b5}'
                | '\u{180b}'..='\u{180d}'
                | '\u{fe00}'..='\u{fe0e}'
                | '\u{ffa0}'
                | '\u{fff9}'..='\u{fffb}'
                | '\u{e0000}'..='\u{e007f}'
        )
}

pub(crate) fn visible(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    push_visible(&mut out, value);
    out
}

fn visible_edges(edges: &[String]) -> String {
    let mut out = String::new();
    for (index, edge) in edges.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        push_visible(&mut out, edge);
    }
    out
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

fn json_str(out: &mut String, level: usize, key: &str, value: &str, comma: bool) {
    indent(out, level);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    push_json_string(out, value);
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn json_num(out: &mut String, level: usize, key: &str, value: u128, comma: bool) {
    indent(out, level);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    out.push_str(&value.to_string());
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn json_bool(out: &mut String, level: usize, key: &str, value: bool, comma: bool) {
    indent(out, level);
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    out.push_str(if value { "true" } else { "false" });
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn json_string_array(out: &mut String, level: usize, key: &str, values: &[String], comma: bool) {
    indent(out, level);
    out.push('"');
    out.push_str(key);
    out.push_str("\": [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(out, value);
    }
    out.push(']');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if needs_escape(other) => {
                let mut units = [0u16; 2];
                for unit in other.encode_utf16(&mut units) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{parse_log, Record};
    use crate::{build_report, default_edges, default_prices, load_logs};
    use std::path::PathBuf;

    fn fixture_report() -> Report {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/twee_regios.log");
        let records = load_logs(&[path]).unwrap();
        build_report(
            &default_prices().unwrap(),
            &default_edges().unwrap(),
            &records,
            None,
        )
        .unwrap()
    }

    #[test]
    fn text_report_says_edge_is_not_the_viewer() {
        let text = render_text(&fixture_report());
        assert!(text.contains(NOTE));
        assert!(text.contains("niet het land van de kijker"));
        assert!(text.contains("/video.mp4"));
        assert!(text.contains("/img.png"));
        assert!(text.contains(EXCLUDES));
    }

    #[test]
    fn text_report_marks_onbekend_unpriced() {
        let text = render_text(&fixture_report());
        assert!(text.contains("onbekend\tZZZ\t50\t0\t1\t0\t-\tnee\n"));
    }

    #[test]
    fn json_escapes_a_quote_in_the_path() {
        let record = Record {
            month: "2026-01".to_string(),
            location: "IAD1".to_string(),
            bytes: 1,
            path: "a\"b".to_string(),
            protocol: "https".to_string(),
        };
        let report = build_report(
            &default_prices().unwrap(),
            &default_edges().unwrap(),
            &[record],
            None,
        )
        .unwrap();
        let json = render_json(&report);
        assert!(json.contains("a\\\"b"));
        assert!(json.contains(
            "\"source_url\": \"https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/\""
        ));
    }

    #[test]
    fn empty_log_says_geen_verkeer() {
        let records = parse_log(
            "leeg",
            "#Fields: date x-edge-location sc-bytes cs-uri-stem cs-protocol\n",
        )
        .unwrap();
        let report = build_report(
            &default_prices().unwrap(),
            &default_edges().unwrap(),
            &records,
            None,
        )
        .unwrap();
        assert!(render_text(&report).contains("geen verkeer\n"));
        assert!(render_json(&report).contains("\"months\": []"));
    }

    #[test]
    fn text_escapes_controls_in_path_and_location() {
        let record = Record {
            month: "2026-01".to_string(),
            location: "\u{1b}[2Jxx".to_string(),
            bytes: 4,
            path: "/a\u{1b}[31mROOD\u{1b}]0;titel\u{7}".to_string(),
            protocol: "https".to_string(),
        };
        let report = build_report(
            &default_prices().unwrap(),
            &default_edges().unwrap(),
            &[record],
            None,
        )
        .unwrap();
        let text = render_text(&report);
        assert!(!text.contains('\u{1b}'));
        assert!(!text.contains('\u{7}'));
        assert!(text.contains("\\u001b[2Jxx"));
        assert!(text.contains("/a\\u001b[31mROOD\\u001b]0;titel\\u0007"));
        let json = render_json(&report);
        assert!(!json.contains('\u{1b}'));
        assert!(!json.contains('\u{7}'));
    }

    #[test]
    fn text_and_json_escape_label_url_and_bidi() {
        let mut prices = default_prices().unwrap();
        prices.source_url = "https://x.test/\u{202e}a".to_string();
        prices.regions[0].label = "L\u{1b}[2J\u{2028}\u{200f}\u{2066}".to_string();
        let record = Record {
            month: "2026-01".to_string(),
            location: "IAD89-C1".to_string(),
            bytes: 4,
            path: "/\u{202e}p".to_string(),
            protocol: "https".to_string(),
        };
        let report = build_report(&prices, &default_edges().unwrap(), &[record], None).unwrap();
        for out in [render_text(&report), render_json(&report)] {
            for bad in ['\u{1b}', '\u{202e}', '\u{2028}', '\u{200f}', '\u{2066}'] {
                assert!(!out.contains(bad), "{bad:?}");
            }
        }
        assert!(render_text(&report).contains("\\u202e"));
    }

    #[test]
    fn text_escapes_invisible_filler_characters() {
        let mut prices = default_prices().unwrap();
        prices.regions[0].label = "L\u{3164}x".to_string();
        let record = Record {
            month: "2026-01".to_string(),
            location: "IAD89-C1".to_string(),
            bytes: 4,
            path: "/a\u{3164}b".to_string(),
            protocol: "https".to_string(),
        };
        let report = build_report(&prices, &default_edges().unwrap(), &[record], None).unwrap();
        let text = render_text(&report);
        assert!(!text.contains('\u{3164}'));
        assert!(text.contains("\\u3164"));
        for ch in ['\u{00ad}', '\u{034f}', '\u{2800}', '\u{3164}', '\u{fe0f}'] {
            assert!(needs_escape(ch), "{ch:?}");
        }
    }

    #[test]
    fn cur_top_hidden_characters_are_escaped() {
        let mut added = vec!['\u{115f}', '\u{1160}', '\u{17b4}', '\u{17b5}', '\u{ffa0}'];
        added.extend('\u{180b}'..='\u{180d}');
        added.extend('\u{fe00}'..='\u{fe0e}');
        added.extend('\u{fff9}'..='\u{fffb}');
        assert_eq!(added.len(), 5 + 3 + 15 + 3);
        for ch in added {
            assert!(needs_escape(ch), "{ch:?}");
            let shown = visible(&format!("a{ch}b"));
            assert!(!shown.contains(ch), "{ch:?}");
            assert!(shown.contains("\\u"), "{ch:?}");
            let mut json = String::new();
            push_json_string(&mut json, &format!("a{ch}b"));
            assert!(!json.contains(ch), "{ch:?}");
        }
    }

    #[test]
    fn tag_characters_are_escaped_in_text_and_json() {
        for ch in ['\u{e0000}', '\u{e0041}', '\u{e007f}'] {
            assert!(needs_escape(ch), "{ch:?}");
        }
        assert!(!needs_escape('\u{e0080}'));
        assert_eq!(visible("a\u{e0041}b"), "a\\u{e0041}b");
        let mut json = String::new();
        push_json_string(&mut json, "a\u{e0041}\u{ad}b");
        assert_eq!(json, "\"a\\udb40\\udc41\\u00adb\"");
    }
}
