use super::WorkSpace;

use rust_covfix::{CoverageFixer, FileCoverage, LineCoverage, PackageCoverage};

macro_rules! line_coveages {
    ($($line:expr => $count:expr),*) => {
        line_coverages![$($line => $count,)*]
    };
    ($($line:expr => $count:expr,)*) => {
        vec![
            $(
                LineCoverage { line_number: $line, count: $count },
            )*
        ]
    }
}

#[test]
fn closing_brackets() {
    let ws = WorkSpace::from_template("./tests/fix");
    let source_file = ws.path().join("closing_brackets.rs");

    let original_line_covs = line_coveages!(
        0 => 1,
        1 => 1,
        2 => 1,
        3 => 1,
        4 => 1,
        5 => 0,
        6 => 0,
        7 => 0,
        9 => 0,
        10 => 0,
    );

    let expected_line_covs = line_coveages!(
        0 => 1,
        1 => 1,
        2 => 1,
        3 => 1,
        4 => 1,
        5 => 0,
        7 => 0,
    );

    let mut coverage = PackageCoverage::new(vec![FileCoverage::new(
        &source_file,
        original_line_covs,
        vec![],
    )]);

    let fixer = CoverageFixer::new().unwrap();
    fixer.fix(&mut coverage).unwrap();

    assert_eq!(
        coverage.file_coverages()[0].line_coverages(),
        &*expected_line_covs
    );
}

#[test]
fn tests_mod() {
    let ws = WorkSpace::from_template("./tests/fix");
    let source_file = ws.path().join("tests_mod.rs");

    let original_line_covs = line_coveages!(
        0 => 1,
        1 => 1,
        2 => 1,
        9 => 1,
        10 => 1,
    );

    let expected_line_covs = line_coveages!(
        0 => 1,
        1 => 1,
    );

    let mut coverage = PackageCoverage::new(vec![FileCoverage::new(
        &source_file,
        original_line_covs,
        vec![],
    )]);

    let fixer = CoverageFixer::new().unwrap();
    fixer.fix(&mut coverage).unwrap();

    assert_eq!(
        coverage.file_coverages()[0].line_coverages(),
        &*expected_line_covs
    );
}
