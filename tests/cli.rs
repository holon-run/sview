use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{fs, process::Command};
use tempfile::tempdir;

#[test]
fn emits_json_for_a_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sample.txt");
    fs::write(&path, "hello\n").unwrap();

    Command::cargo_bin("sview")
        .unwrap()
        .args([path.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"language\": \"unknown\""))
        .stdout(predicate::str::contains("\"kind\": \"file\""));
}

#[test]
fn emits_text_by_default() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("README.md");
    fs::write(&path, "# Title\n").unwrap();

    Command::cargo_bin("sview")
        .unwrap()
        .arg(path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("README.md (markdown)"))
        .stdout(predicate::str::contains("└─ heading Title L1-1"));
}
