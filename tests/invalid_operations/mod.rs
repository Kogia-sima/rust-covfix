use super::WorkSpace;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use rust_covfix::{error::ErrorKind, parser::LcovParser, CoverageReader};

#[test]
fn root_is_not_a_dir() {
    let ws = WorkSpace::from_template("tests/invalid_operations");

    let mut exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if cfg!(windows) {
        exe.push("target\\debug\\rust-covfix");
    } else {
        exe.push("target/debug/rust-covfix");
    }

    let status = Command::new(exe)
        .arg("--root")
        .arg("not_a_directory")
        .arg(ws.path().join("lcov_empty.info"))
        .stderr(Stdio::null())
        .status()
        .unwrap();

    assert!(!status.success());
}

#[test]
fn input_is_not_a_file() {
    let ws = WorkSpace::from_template("tests/invalid_operations");

    let mut exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if cfg!(windows) {
        exe.push("target\\debug\\rust-covfix");
    } else {
        exe.push("target/debug/rust-covfix");
    }

    let status = Command::new(exe)
        .arg(ws.path().join("lcov_not_exists.info"))
        .stderr(Stdio::null())
        .status()
        .unwrap();

    assert!(!status.success());
}

#[test]
fn source_file_not_found() {
    let ws = WorkSpace::from_template("tests/invalid_operations");

    let parser = LcovParser::new(ws.path());
    let result = parser.read_from_file(&ws.path().join("lcov.info"));

    assert_matches!(result, Err(_));
    assert_matches!(result.unwrap_err().kind(), ErrorKind::SourceFileNotFound(_));
}

#[test]
fn target_dir_not_found() {
    let ws = WorkSpace::from_template("tests/invalid_operations");

    let mut exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if cfg!(windows) {
        exe.push("target\\debug\\rust-covfix");
    } else {
        exe.push("target/debug/rust-covfix");
    }

    let status = Command::new(exe)
        .arg(ws.path().join("lcov_empty.info"))
        .current_dir(ws.path().ancestors().last().unwrap())
        .stderr(Stdio::null())
        .status()
        .unwrap();

    assert!(!status.success());
}
