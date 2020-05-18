use super::WorkSpace;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
fn all_rules() {
    let ws = WorkSpace::from_template("tests/multiple_files");

    let lcov1 = ws.path().join("lcov.info");
    let lcov2 = ws.path().join("lcov2.info");
    let lcov3 = ws.path().join("lcov3.info");

    let mut exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if cfg!(windows) {
        exe.push("target\\debug\\rust-covfix");
    } else {
        exe.push("target/debug/rust-covfix");
    }

    let result = Command::new(exe)
        .current_dir(ws.path().join("src"))
        .arg("-o")
        .arg(&lcov3)
        .arg(&lcov1)
        .output()
        .unwrap();

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stderr).unwrap(),
        r"Coverages are fixed successfully!
  line:   100.00% (10 of 10 lines)    => 100.00% (5 of 5 lines)
  branch: NaN% (0 of 0 branches) => NaN% (0 of 0 branches)

"
    );

    let expected_content = fs::read_to_string(lcov2).unwrap();
    let content = fs::read_to_string(lcov3).unwrap();
    assert_eq!(content, expected_content);
}

#[test]
fn invalid() {
    let ws = WorkSpace::from_template("tests/multiple_files");

    let lcov = ws.path().join("lcov_invalid.info");

    let mut exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if cfg!(windows) {
        exe.push("target\\debug\\rust-covfix");
    } else {
        exe.push("target/debug/rust-covfix");
    }

    let result = Command::new(exe)
        .current_dir(ws.path().join("src"))
        .arg(&lcov)
        .output()
        .unwrap();

    assert!(!result.status.success());
    assert_eq!(
        String::from_utf8(result.stderr)
            .unwrap()
            .lines()
            .next()
            .unwrap(),
        "Error: Failed to fix coverage"
    );
}
