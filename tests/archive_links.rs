//! The release archive must be self-contained: every relative link in the documents that the
//! `Package` step of `.github/workflows/release.yml` copies must point at something that the
//! same step copies too.
//!
//! The test reads the `cp` lines of that step, so it follows the workflow. The published crate
//! leaves out `.github/`; there the test has nothing to check and returns.

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The paths (relative to the repository root) that the `Package` step copies into the archive.
/// The binary and the generated license file are not in the repository; they are returned too.
fn packaged_paths(workflow: &str) -> Vec<String> {
    let mut in_step = false;
    let mut paths = Vec::new();
    for line in workflow.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- name:") {
            in_step = trimmed == "- name: Package";
            continue;
        }
        if !in_step || !trimmed.starts_with("cp ") {
            continue;
        }
        let words: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
        // The last word is the destination; flags such as -R are not paths.
        for word in &words[..words.len() - 1] {
            if !word.starts_with('-') {
                paths.push(word.trim_matches('"').to_string());
            }
        }
    }
    paths
}

fn in_archive(archive: &[String], path: &str) -> bool {
    archive.iter().any(|entry| {
        path == entry || path.starts_with(&format!("{}/", entry.trim_end_matches('/')))
    })
}

fn markdown_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let mut entries: Vec<PathBuf> = fs::read_dir(path)
            .expect("read dir")
            .map(|entry| entry.expect("entry").path())
            .collect();
        entries.sort();
        for entry in entries {
            markdown_files(&entry, out);
        }
    } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
        out.push(path.to_path_buf());
    }
}

/// Targets of `[text](target)` that are relative (no scheme, no anchor-only).
fn link_targets(text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("](") {
        rest = &rest[at + 2..];
        let Some(end) = rest.find(')') else { break };
        let target = rest[..end].split_whitespace().next().unwrap_or("");
        let target = target.split('#').next().unwrap_or("");
        if !target.is_empty() && !target.contains("://") && !target.starts_with("mailto:") {
            targets.push(target.to_string());
        }
        rest = &rest[end..];
    }
    targets
}

/// Names of a file at the top level (SECURITY.md) or under `data/`, in backticks or in plain
/// text, which the README sends the reader to. A name counts only when it exists in the
/// repository.
fn named_files(text: &str, base: &Path) -> Vec<String> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || "_./-".contains(c)))
        .map(|token| token.trim_end_matches('.'))
        .filter(|token| {
            let top_md = !token.contains('/') && token.ends_with(".md");
            (top_md || token.starts_with("data/")) && base.join(token).exists()
        })
        .map(str::to_string)
        .collect()
}

#[test]
fn every_relative_link_in_the_packaged_documents_points_into_the_archive() {
    let base = root();
    let Ok(workflow) = fs::read_to_string(base.join(".github/workflows/release.yml")) else {
        return;
    };
    let archive = packaged_paths(&workflow);
    assert!(!archive.is_empty(), "no cp line found in the Package step");

    let mut docs = Vec::new();
    for entry in &archive {
        let path = base.join(entry);
        if path.exists() {
            markdown_files(&path, &mut docs);
        }
    }
    assert!(docs.len() >= 4, "too few documents: {docs:?}");

    let mut missing = Vec::new();
    for doc in docs {
        let rel = doc
            .strip_prefix(&base)
            .expect("inside the repo")
            .to_path_buf();
        let dir = rel.parent().unwrap_or(Path::new(""));
        let text = fs::read_to_string(&doc).expect("read document");
        let mut targets = link_targets(&text);
        let name = rel.to_string_lossy();
        if name == "README.md" || name == "README.nl.md" {
            // Named files are relative to the archive root, like the README itself.
            targets.extend(named_files(&text, &base));
        }
        for target in targets {
            let joined = dir.join(&target);
            let normal: PathBuf = joined.components().collect();
            let wanted = normal.to_string_lossy().replace('\\', "/");
            if !in_archive(&archive, &wanted) {
                missing.push(format!("{name} -> {target}"));
            }
        }
    }
    missing.sort();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "not in the release archive: {missing:#?}"
    );
}
