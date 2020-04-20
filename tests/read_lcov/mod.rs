use super::WorkSpace;

use rust_covfix::{
    parser::LcovParser, BranchCoverage, CoverageReader, FileCoverage, LineCoverage, PackageCoverage,
};

#[test]
fn simple() {
    let ws = WorkSpace::from_template("tests/read_lcov");
    let lcov_file = ws.path().join("lcov.info");

    let parser = LcovParser::new(ws.path());
    let coverage = parser.read_from_file(&lcov_file).unwrap();

    let expected_coverage = PackageCoverage::new(vec![
        FileCoverage::new(
            ws.path().join("src/main.rs"),
            vec![
                LineCoverage {
                    line_number: 3,
                    count: Some(1),
                },
                LineCoverage {
                    line_number: 4,
                    count: Some(6),
                },
                LineCoverage {
                    line_number: 5,
                    count: Some(5),
                },
                LineCoverage {
                    line_number: 7,
                    count: Some(1),
                },
            ],
            vec![
                BranchCoverage {
                    line_number: 4,
                    block_number: Some(0),
                    taken: Some(true),
                },
                BranchCoverage {
                    line_number: 4,
                    block_number: Some(0),
                    taken: Some(false),
                },
                BranchCoverage {
                    line_number: 4,
                    block_number: Some(0),
                    taken: Some(true),
                },
            ],
        ),
        FileCoverage::new(
            ws.path().join("src/sub.rs"),
            vec![
                LineCoverage {
                    line_number: 1,
                    count: Some(5),
                },
                LineCoverage {
                    line_number: 2,
                    count: Some(5),
                },
                LineCoverage {
                    line_number: 3,
                    count: Some(5),
                },
            ],
            vec![],
        ),
    ]);

    assert_eq!(coverage, expected_coverage);
}

#[test]
fn empty() {
    let ws = WorkSpace::from_template("tests/read_lcov");
    let lcov_file = ws.path().join("lcov_empty.info");

    let parser = LcovParser::new(ws.path());
    let coverage = parser.read_from_file(&lcov_file).unwrap();

    let expected_coverage = PackageCoverage::new(vec![]);

    assert_eq!(coverage, expected_coverage);
}
