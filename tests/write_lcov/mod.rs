use super::WorkSpace;
use std::fs;
use std::io::Cursor;

use rust_covfix::{
    lcov::LcovParser, BranchCoverage, CoverageWriter, FileCoverage, LineCoverage, PackageCoverage,
};

#[test]
fn test() {
    let ws = WorkSpace::from_template("tests/write_lcov");
    let lcov_file = ws.path().join("lcov.info");
    let expected_content = fs::read_to_string(&lcov_file).unwrap();

    let coverage = PackageCoverage::with_test_name(
        "write_lcov",
        vec![
            FileCoverage::new(
                ws.path().join("src/lib.rs"),
                vec![
                    LineCoverage {
                        line_number: 1,
                        count: 2,
                    },
                    LineCoverage {
                        line_number: 2,
                        count: 2,
                    },
                    LineCoverage {
                        line_number: 7,
                        count: 1,
                    },
                    LineCoverage {
                        line_number: 8,
                        count: 4,
                    },
                    LineCoverage {
                        line_number: 9,
                        count: 1,
                    },
                ],
                vec![
                    BranchCoverage {
                        line_number: Some(2),
                        block_number: Some(0),
                        branch_number: Some(0),
                        taken: false,
                    },
                    BranchCoverage {
                        line_number: Some(2),
                        block_number: Some(0),
                        branch_number: Some(1),
                        taken: false,
                    },
                    BranchCoverage {
                        line_number: Some(2),
                        block_number: Some(0),
                        branch_number: Some(2),
                        taken: true,
                    },
                    BranchCoverage {
                        line_number: Some(7),
                        block_number: Some(0),
                        branch_number: Some(0),
                        taken: true,
                    },
                    BranchCoverage {
                        line_number: Some(7),
                        block_number: Some(0),
                        branch_number: Some(1),
                        taken: false,
                    },
                ],
            ),
            FileCoverage::new(
                ws.path().join("src/sub.rs"),
                vec![
                    LineCoverage {
                        line_number: 0,
                        count: 1,
                    },
                    LineCoverage {
                        line_number: 1,
                        count: 2,
                    },
                    LineCoverage {
                        line_number: 2,
                        count: 3,
                    },
                    LineCoverage {
                        line_number: 8,
                        count: 0,
                    },
                    LineCoverage {
                        line_number: 9999,
                        count: 3,
                    },
                ],
                vec![],
            ),
        ],
    );

    let buffer: Vec<u8> = Vec::with_capacity(expected_content.len() + 1);
    let mut writer = Cursor::new(buffer);

    let parser = LcovParser::new(ws.path());
    parser.write(&coverage, &mut writer).unwrap();

    let content = String::from_utf8(writer.into_inner()).unwrap();
    assert_eq!(content.trim_end(), expected_content.trim_end());
}
