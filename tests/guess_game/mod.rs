use super::WorkSpace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test() {
    let ws = WorkSpace::from_template("tests/guess_game");

    let lcov1 = ws.path().join("lcov.info");
    let lcov2 = ws.path().join("lcov2.info");
    let lcov3 = ws.path().join("lcov3.info");

    let mut exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if cfg!(windows) {
        exe.push("target\\debug\\rust-covfix");
    } else {
        exe.push("target/debug/rust-covfix");
    }

    let status = Command::new(exe)
        .arg("--root")
        .arg(ws.path())
        .arg("-o")
        .arg(&lcov3)
        .arg(&lcov1)
        .status()
        .unwrap();

    assert!(status.success());

    let expected_content = fs::read_to_string(lcov2).unwrap();
    let content = fs::read_to_string(lcov3).unwrap();

    assert_eq!(content, expected_content);
}
