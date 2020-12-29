use super::WorkSpace;
use pretty_assertions::assert_eq;
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
                        line_number: 2,
                        count: Some(2),
                    },
                    LineCoverage {
                        line_number: 3,
                        count: Some(2),
                    },
                    LineCoverage {
                        line_number: 8,
                        count: Some(1),
                    },
                    LineCoverage {
                        line_number: 9,
                        count: Some(4),
                    },
                    LineCoverage {
                        line_number: 10,
                        count: Some(1),
                    },
                ],
                vec![
                    BranchCoverage {
                        line_number: 3,
                        block_number: Some(0),
                        taken: Some(false),
                    },
                    BranchCoverage {
                        line_number: 3,
                        block_number: Some(0),
                        taken: Some(false),
                    },
                    BranchCoverage {
                        line_number: 3,
                        block_number: Some(0),
                        taken: Some(true),
                    },
                    BranchCoverage {
                        line_number: 8,
                        block_number: Some(0),
                        taken: Some(true),
                    },
                    BranchCoverage {
                        line_number: 8,
                        block_number: Some(0),
                        taken: Some(false),
                    },
                ],
            ),
            FileCoverage::new(
                ws.path().join("src/sub.rs"),
                vec![
                    LineCoverage {
                        line_number: 1,
                        count: Some(1),
                    },
                    LineCoverage {
                        line_number: 2,
                        count: Some(2),
                    },
                    LineCoverage {
                        line_number: 3,
                        count: Some(3),
                    },
                    LineCoverage {
                        line_number: 9,
                        count: Some(0),
                    },
                    LineCoverage {
                        line_number: 10000,
                        count: Some(3),
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
