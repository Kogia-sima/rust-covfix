use super::WorkSpace;

use rust_covfix::{
    parser::LcovParser, BranchCoverage, CoverageReader, FileCoverage, LineCoverage, PackageCoverage,
};

#[test]
fn test() {
    let ws = WorkSpace::from_template("tests/read_lcov");
    let lcov_file = ws.path().join("lcov.info");

    let parser = LcovParser::new(ws.path());
    let coverage = parser.read_from_file(&lcov_file).unwrap();

    let expected_coverage = PackageCoverage::new(vec![
        FileCoverage::new(
            ws.path().join("src/main.rs"),
            vec![
                LineCoverage {
                    line_number: 2,
                    count: 1,
                },
                LineCoverage {
                    line_number: 3,
                    count: 6,
                },
                LineCoverage {
                    line_number: 4,
                    count: 5,
                },
                LineCoverage {
                    line_number: 6,
                    count: 1,
                },
            ],
            vec![
                BranchCoverage {
                    line_number: Some(3),
                    block_number: Some(0),
                    taken: true,
                },
                BranchCoverage {
                    line_number: Some(3),
                    block_number: Some(0),
                    taken: false,
                },
                BranchCoverage {
                    line_number: Some(3),
                    block_number: Some(0),
                    taken: true,
                },
            ],
        ),
        FileCoverage::new(
            ws.path().join("src/sub.rs"),
            vec![
                LineCoverage {
                    line_number: 0,
                    count: 5,
                },
                LineCoverage {
                    line_number: 1,
                    count: 5,
                },
                LineCoverage {
                    line_number: 2,
                    count: 5,
                },
            ],
            vec![],
        ),
    ]);

    assert_eq!(coverage, expected_coverage);
}
