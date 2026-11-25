use std::fs;
use std::path::{Path, PathBuf};

fn read(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

fn stated_passed(text: &str) -> usize {
    let line = text
        .lines()
        .find(|line| line.contains("passed tests"))
        .expect("passed tests");
    let number = line
        .split("gave ")
        .nth(1)
        .expect("gave")
        .split_whitespace()
        .next()
        .expect("getal");
    number.parse().expect("passed tests is a number")
}

fn suite_count(text: &str) -> usize {
    let line = text
        .lines()
        .find(|line| line.contains("SUITE_TESTS:"))
        .expect("SUITE_TESTS");
    let number = line
        .split("SUITE_TESTS:")
        .nth(1)
        .expect("getal")
        .split_whitespace()
        .next()
        .expect("getal");
    number.parse().expect("SUITE_TESTS is een getal")
}

fn count_tests(root: &Path) -> usize {
    let mut count = 0;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display())) {
            let path = entry.expect("mapregel").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let text =
                fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
            count += text
                .lines()
                .filter(|line| line.trim_start().starts_with("#[test]"))
                .count();
        }
    }
    count
}

fn claim_names(readme: &str) -> Vec<String> {
    let after = readme.split("## Claims").nth(1).expect("## Claims");
    let section = after.split("\n## ").next().expect("sectie");
    let mut names = Vec::new();
    let mut rest = section;
    while let Some(start) = rest.find('`') {
        rest = &rest[start + 1..];
        let Some(end) = rest.find('`') else {
            break;
        };
        let token = &rest[..end];
        rest = &rest[end + 1..];
        let ident = !token.is_empty()
            && token
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            && token
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphabetic());
        if ident {
            names.push(token.to_string());
        }
    }
    names
}

fn defines(text: &str, name: &str) -> bool {
    let needle = format!("fn {name}(");
    text.lines()
        .any(|line| line.trim_start().starts_with(&needle))
}

#[test]
fn readme_lists_source_url_and_check_date() {
    let readme = read("README.md");
    assert!(readme.contains("https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/"));
    assert!(readme.contains("2026-11-09"));
    assert!(readme.contains("sc-bytes"));
    assert!(readme.contains("x-edge-location"));
    assert!(readme.contains("not the viewer's country"));
    assert!(readme.contains("onbekend"));
    assert!(readme.contains("Savings Bundle"));
    assert!(readme.contains("10000000"));
    assert!(readme.contains("1 TB"));
    assert!(readme.contains("0.085"));
}

#[test]
fn readme_claims_name_real_tests() {
    let names = claim_names(&read("README.md"));
    assert!(!names.is_empty());
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut sources = String::new();
    for dir in ["src", "tests"] {
        let mut stack = vec![root.join(dir)];
        while let Some(current) = stack.pop() {
            for entry in fs::read_dir(&current).expect("map") {
                let path = entry.expect("mapregel").path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                    sources.push_str(&fs::read_to_string(&path).expect("bron"));
                    sources.push('\n');
                }
            }
        }
    }
    for name in names {
        assert!(defines(&sources, &name), "geen fn {name}");
    }
}

#[test]
fn quality_and_readme_share_suite_count() {
    let from_readme = suite_count(&read("README.md"));
    let from_quality = suite_count(&read("QUALITY.md"));
    assert_eq!(from_readme, from_quality);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let counted = count_tests(&root.join("src")) + count_tests(&root.join("tests"));
    assert_eq!(from_readme, counted);
    assert_eq!(stated_passed(&read("QUALITY.md")), counted);
}

#[test]
fn manifest_has_no_aws_sdk() {
    let manifest = read("Cargo.toml");
    for forbidden in ["aws-sdk", "rusoto", "reqwest", "hyper", "ureq"] {
        assert!(!manifest.contains(forbidden), "{forbidden}");
    }
}

#[test]
fn changelog_has_0_1_0() {
    let log = read("CHANGELOG.md");
    assert!(log.contains("## [1.0.0] - 2026-11-15"));
    assert!(log.contains("0.1.0"));
    assert!(log.contains("## [0.1.0] - "));
}

#[test]
fn licenses_match_the_manifest() {
    let manifest = read("Cargo.toml");
    assert!(manifest.contains("MIT OR Apache-2.0"));
    let mit = read("LICENSE-MIT");
    let apache = read("LICENSE-APACHE");
    assert!(mit.contains("MIT License"));
    assert!(mit.contains("PEPAI"));
    assert!(mit.contains("2026"));
    assert!(apache.contains("Apache License"));
    assert!(apache.contains("Version 2.0"));
    assert!(apache.contains("PEPAI"));
    assert!(apache.contains("2026"));
}
