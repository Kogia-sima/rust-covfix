use super::WorkSpace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn all_rules() {
    let ws = WorkSpace::from_template("tests/workspace");

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
        .current_dir(ws.path().join("covfix-test1").join("src"))
        .arg("-o")
        .arg(&lcov3)
        .arg(&lcov1)
        .output()
        .unwrap();

    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stderr).unwrap(),
        r"Coverages are fixed successfully!
  line:   0.00% (0 of 7 lines)    => 0.00% (0 of 6 lines)
  branch: 0.00% (0 of 12 branches) => 0.00% (0 of 12 branches)

"
    );

    let expected_content = fs::read_to_string(lcov2).unwrap();
    let content = fs::read_to_string(lcov3).unwrap();

    assert_eq!(content, expected_content);
}
