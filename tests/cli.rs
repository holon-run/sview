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

#[test]
fn emits_text_for_multiple_files() {
    let dir = tempdir().unwrap();
    let readme = dir.path().join("README.md");
    let lib = dir.path().join("lib.rs");
    fs::write(&readme, "# Title\n").unwrap();
    fs::write(&lib, "pub struct Client;\n").unwrap();

    Command::cargo_bin("sview")
        .unwrap()
        .args([readme.to_str().unwrap(), lib.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("README.md (markdown)"))
        .stdout(predicate::str::contains("lib.rs (rust)"))
        .stdout(predicate::str::contains("└─ struct Client L1-1"));
}

#[test]
fn emits_json_array_for_multiple_files() {
    let dir = tempdir().unwrap();
    let readme = dir.path().join("README.md");
    let lib = dir.path().join("lib.rs");
    fs::write(&readme, "# Title\n").unwrap();
    fs::write(&lib, "pub struct Client;\n").unwrap();

    Command::cargo_bin("sview")
        .unwrap()
        .args([readme.to_str().unwrap(), lib.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["))
        .stdout(predicate::str::contains("\"language\": \"markdown\""))
        .stdout(predicate::str::contains("\"language\": \"rust\""));
}

#[test]
fn emits_text_for_typescript() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("app.ts");
    fs::write(
        &path,
        "export interface User { id: string }\nexport function loadUser(): User { return { id: '1' }; }\n",
    )
    .unwrap();

    Command::cargo_bin("sview")
        .unwrap()
        .arg(path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("app.ts (typescript)"))
        .stdout(predicate::str::contains("├─ interface User L1-1"))
        .stdout(predicate::str::contains("└─ function loadUser L2-2"));
}

#[test]
fn emits_text_for_java() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("MainActivity.java");
    fs::write(
        &path,
        "package com.example;\n\npublic class MainActivity {\n  void onCreate() {}\n}\n",
    )
    .unwrap();

    Command::cargo_bin("sview")
        .unwrap()
        .arg(path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("MainActivity.java (java)"))
        .stdout(predicate::str::contains("├─ package com.example L1-1"))
        .stdout(predicate::str::contains("└─ class MainActivity L3-5"))
        .stdout(predicate::str::contains("method onCreate L4-4"));
}
