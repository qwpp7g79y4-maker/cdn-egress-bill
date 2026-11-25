//! Debug builds from a git checkout of this crate print the short commit after the
//! version. A release build, a build from an unpacked package, or a failing git
//! leaves the version as it is: nothing here invents a commit.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(cdn_git_sha)");
    println!("cargo:rerun-if-changed=build.rs");
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    // Only this crate's own checkout counts, never a repository around an unpacked package.
    if !root.join(".git").exists() {
        return;
    }
    track(&root, "HEAD");
    // The branch ref is found through git, because in a worktree `.git` is a file and
    // the branch lives in the common directory, not in this worktree's own HEAD file.
    if let Some(head) = git(&root, &["symbolic-ref", "--quiet", "HEAD"]) {
        track(&root, &head);
        track(&root, "packed-refs");
    }
    if std::env::var("PROFILE").unwrap_or_default() != "debug" {
        return;
    }
    if let Some(sha) = git(&root, &["rev-parse", "--verify", "HEAD"]) {
        if sha.len() == 40 && sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            println!("cargo:rustc-cfg=cdn_git_sha");
            println!("cargo:rustc-env=CDN_GIT_SHA={}", &sha[..7]);
        }
    }
}

/// Rebuild when the commit changes (the path comes from git, so a worktree works too).
fn track(root: &PathBuf, name: &str) {
    if let Some(path) = git(root, &["rev-parse", "--git-path", name]) {
        let path = PathBuf::from(path);
        let path = if path.is_absolute() {
            path
        } else {
            root.join(path)
        };
        if path.is_file() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

fn git(root: &PathBuf, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8(out.stdout).ok()?.trim().to_string())
}
