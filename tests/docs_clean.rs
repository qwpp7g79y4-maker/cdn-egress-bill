//! The published files must not carry local paths, build-wrapper options, working-note
//! file names or the names of internal tools.
//!
//! Every word is written in pieces, so that this file does not contain them either.

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN: [&str; 17] = [
    concat!("/me", "dia/"),
    concat!("/ho", "me/"),
    concat!("~", "/"),
    concat!("--max", "-gb"),
    concat!("--na", "am"),
    concat!("task", ".md"),
    concat!("blueprint", ".md"),
    concat!("geheugen", "wachter"),
    concat!("bouw", "-tmp"),
    concat!("gr", "ok"),
    concat!("cla", "ude"),
    concat!("anthr", "opic"),
    concat!("co-auth", "ored"),
    concat!("son", "net"),
    concat!("op", "us"),
    concat!("open", "bot"),
    concat!("brug", "-prompts"),
];

/// Every file under `path`. A path that does not exist is skipped: the published package
/// leaves out `.github/` and `.gitattributes`, and this test runs there too.
fn collect(path: &Path, out: &mut Vec<PathBuf>) {
    if !path.exists() {
        return;
    }
    if path.is_dir() {
        let mut entries: Vec<PathBuf> = fs::read_dir(path)
            .unwrap_or_else(|e| panic!("read dir {}: {e}", path.display()))
            .map(|entry| entry.expect("entry").path())
            .collect();
        entries.sort();
        for entry in entries {
            collect(&entry, out);
        }
    } else {
        out.push(path.to_path_buf());
    }
}

/// The words found in `text`, each with its 1-based line number. Case does not matter.
fn hits(text: &str) -> Vec<(usize, &'static str)> {
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let lower = line.to_lowercase();
        for word in FORBIDDEN {
            if lower.contains(word) {
                found.push((index + 1, word));
            }
        }
    }
    found
}

fn problems_in(files: &[PathBuf]) -> Vec<String> {
    let mut problems = Vec::new();
    for file in files {
        let text =
            fs::read_to_string(file).unwrap_or_else(|e| panic!("read {}: {e}", file.display()));
        for (line, word) in hits(&text) {
            problems.push(format!("{}:{line}: {word}", file.display()));
        }
    }
    problems
}

#[test]
fn public_docs_have_no_internal_names_or_local_paths() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for name in [
        "README.md",
        "QUALITY.md",
        "CHANGELOG.md",
        "SECURITY.md",
        "THIRD-PARTY-LICENSES.html",
    ] {
        files.push(root.join(name));
    }
    collect(&root.join("man"), &mut files);
    assert!(files.len() >= 5, "man/ holds no page: {files:?}");
    let problems = problems_in(&files);
    assert!(
        problems.is_empty(),
        "internal text in public files:\n{}",
        problems.join("\n")
    );
}

/// `.gitignore` is not read: it has to name the two working notes to keep them out of git.
#[test]
fn package_metadata_workflows_sources_and_tests_have_no_internal_names_either() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for name in [
        "Cargo.toml",
        "Cargo.lock",
        "deny.toml",
        ".gitattributes",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        ".github",
        "src",
    ] {
        collect(&root.join(name), &mut files);
    }
    for dir in ["tests", "examples", "data", "scripts"] {
        collect(&root.join(dir), &mut files);
    }
    assert!(files.len() >= 20, "too few files found: {files:?}");
    // A folder that is there must have been read. The package leaves out `.github/`.
    for (dir, file) in [
        (".github", "ci.yml"),
        ("src", "main.rs"),
        ("tests", "docs_clean.rs"),
        ("examples", "access.log"),
        ("data", "prices.txt"),
    ] {
        if root.join(dir).exists() {
            assert!(
                files.iter().any(|path| path.ends_with(file)),
                "{dir}/ exists but {file} was not read: {files:?}"
            );
        }
    }
    let problems = problems_in(&files);
    assert!(
        problems.is_empty(),
        "internal text in package files:\n{}",
        problems.join("\n")
    );
}

#[test]
fn a_path_that_is_not_there_is_skipped() {
    let mut files = Vec::new();
    collect(Path::new("/this/path/does/not/exist"), &mut files);
    assert!(files.is_empty(), "{files:?}");
}

#[test]
fn the_check_itself_finds_each_forbidden_word() {
    for word in FORBIDDEN {
        let text = format!("fine line\nbefore {word} after\n");
        assert_eq!(hits(&text), vec![(2, word)], "{word}");
        let upper = format!("fine line\nbefore {} after\n", word.to_uppercase());
        assert_eq!(hits(&upper), vec![(2, word)], "{word} in capitals");
    }
    assert!(hits("plain text, a `cargo test` command, ./target/release/x").is_empty());
}

/// The text of one `## ` section of a README, up to the next `## ` heading.
fn section<'a>(text: &'a str, heading: &str) -> Option<&'a str> {
    let marker = format!("\n## {heading}\n");
    let start = text.find(&marker)? + marker.len();
    let rest = &text[start..];
    Some(rest.split("\n## ").next().unwrap_or(rest))
}

#[test]
fn both_readmes_have_the_prices_and_maintenance_sections() {
    let prices = fs::read_to_string("data/prices.txt").expect("data/prices.txt");
    let date = prices
        .lines()
        .find_map(|l| l.strip_prefix("checked_on "))
        .expect("checked_on")
        .trim();
    let url = prices
        .lines()
        .find_map(|l| l.strip_prefix("source_url "))
        .expect("source_url")
        .trim();
    for (file, price_head, maint_head, issues) in [
        (
            "README.md",
            "Prices and responsibility",
            "Maintenance",
            "GitHub",
        ),
        (
            "README.nl.md",
            "Prijzen en verantwoordelijkheid",
            "Onderhoud",
            "GitHub",
        ),
    ] {
        let text = format!("\n{}", fs::read_to_string(file).expect(file));
        let p = section(&text, price_head).unwrap_or_else(|| panic!("{file}: no '{price_head}'"));
        assert!(p.contains(date), "{file}: the price date {date} is missing");
        assert!(p.contains(url), "{file}: the source url is missing");
        assert!(p.contains("--prices"), "{file}: --prices is not mentioned");
        let m = section(&text, maint_head).unwrap_or_else(|| panic!("{file}: no '{maint_head}'"));
        assert!(
            m.contains(issues) && m.contains("/issues"),
            "{file}: issues"
        );
        // The two sections sit directly after the limitations section.
        let order: Vec<&str> = text.lines().filter_map(|l| l.strip_prefix("## ")).collect();
        let at = order
            .iter()
            .position(|h| *h == price_head)
            .expect("price heading");
        assert!(
            order[at - 1] == "Limitations" || order[at - 1] == "Beperkingen",
            "{file}: not right after the limitations: {order:?}"
        );
        assert_eq!(order[at + 1], maint_head, "{file}: order {order:?}");
    }
}
