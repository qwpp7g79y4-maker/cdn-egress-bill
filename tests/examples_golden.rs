//! Every command in `examples/README.md` gives the expected output stored next to it.

use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run(extra: &[&str]) -> Vec<u8> {
    let out = Command::new(env!("CARGO_BIN_EXE_cdn-egress-bill"))
        .current_dir(root())
        .args(extra)
        .arg("examples/access.log")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{extra:?}");
    out.stdout
}

fn expected(name: &str) -> Vec<u8> {
    std::fs::read(root().join("examples").join(name))
        .unwrap_or_else(|err| panic!("examples/{name}: {err}"))
}

#[test]
fn each_example_command_gives_its_expected_output() {
    assert_eq!(run(&[]), expected("access.report.txt"));
    assert_eq!(run(&["--md"]), expected("access.report.md"));
    assert_eq!(run(&["--format", "json"]), expected("access.report.json"));
}

#[test]
fn the_examples_readme_names_every_command_and_file() {
    let text = std::fs::read_to_string(root().join("examples/README.md")).unwrap();
    for needle in [
        "cdn-egress-bill examples/access.log",
        "cdn-egress-bill --md examples/access.log",
        "cdn-egress-bill --format json examples/access.log",
        "examples/access.report.txt",
        "examples/access.report.md",
        "examples/access.report.json",
    ] {
        assert!(text.contains(needle), "examples/README.md lacks {needle}");
    }
}
