//! `--version` names the crate version, and a debug build from a git checkout adds the commit.

use std::path::PathBuf;
use std::process::Command;

fn version_line() -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_cdn-egress-bill"))
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    String::from_utf8(out.stdout).unwrap()
}

fn checkout_commit() -> Option<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Only a checkout of this crate counts, never a repository around an unpacked package.
    if !root.join(".git").exists() {
        return None;
    }
    let out = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(&root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let sha = text.trim().to_string();
    (out.status.success() && sha.len() == 40).then_some(sha)
}

#[test]
fn version_is_the_crate_version_and_in_a_debug_checkout_also_the_commit() {
    let line = version_line();
    let base = format!("cdn-egress-bill {}", env!("CARGO_PKG_VERSION"));
    match (cfg!(debug_assertions), checkout_commit()) {
        (true, Some(sha)) => assert_eq!(line, format!("{base} ({})\n", &sha[..7])),
        _ => assert_eq!(line, format!("{base}\n")),
    }
}
