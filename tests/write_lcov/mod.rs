use super::WorkSpace;
use std::fs;

use rust_covfix::{
    parser::LcovParser, BranchCoverage, CoverageWriter, FileCoverage, LineCoverage, PackageCoverage,
};

#[test]
fn test() {
    let ws = WorkSpace::from_template("tests/write_lcov");
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
                        taken: false,
                    },
                    BranchCoverage {
                        line_number: Some(2),
                        block_number: Some(0),
                        taken: false,
                    },
                    BranchCoverage {
                        line_number: Some(2),
                        block_number: Some(0),
                        taken: true,
                    },
                    BranchCoverage {
                        line_number: Some(7),
                        block_number: Some(0),
                        taken: true,
                    },
                    BranchCoverage {
                        line_number: Some(7),
                        block_number: Some(0),
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

    let parser = LcovParser::new(ws.path());
    let target_file = ws.path().join("lcov2.info");
    parser.write_to_file(&coverage, &target_file).unwrap();

    let content = fs::read_to_string(target_file).unwrap();

    let lcov_file = ws.path().join("lcov.info");
    let expected_content = fs::read_to_string(&lcov_file).unwrap();

    assert_eq!(content.trim_end(), expected_content.trim_end());
}
