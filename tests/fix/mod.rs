use super::WorkSpace;

use rust_covfix::{BranchCoverage, CoverageFixer, FileCoverage, LineCoverage, PackageCoverage};

macro_rules! line_coverages {
    () => {
        Vec::<LineCoverage>::new()
    };
    ($($line:expr => $count:expr),*) => {
        line_coverages![$($line => $count,)*]
    };
    ($($line:expr => $count:expr,)*) => {
        vec![
            $(
                LineCoverage { line_number: $line, count: Some($count) },
            )*
        ]
    }
}

macro_rules! branch_coverages {
    () => {
        Vec::<BranchCoverage>::new()
    };
    ($($line:expr => $taken:expr),*) => {
        branch_coverages![$($line => $taken,)*]
    };
    ($($line:expr => $taken:expr,)*) => {
        vec![
            $(
                BranchCoverage { line_number: $line, block_number: None, taken: Some($taken) },
            )*
        ]
    }
}

#[test]
fn closing_brackets() {
    let ws = WorkSpace::from_template("./tests/fix");
    let source_file = ws.path().join("closing_brackets.rs");

    let original_line_covs = line_coverages!(
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

    let expected_line_covs = line_coverages!(
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

    let fixer = CoverageFixer::new();
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

    let original_line_covs = line_coverages!(
        0 => 1,
        1 => 1,
        2 => 1,
        11 => 1,
        12 => 1,
        20 => 1,
        21 => 1
    );

    let expected_line_covs = line_coverages!(
        0 => 1,
        1 => 1,
        20 => 1,
        21 => 1
    );

    let mut coverage = PackageCoverage::new(vec![FileCoverage::new(
        &source_file,
        original_line_covs,
        vec![],
    )]);

    let fixer = CoverageFixer::new();
    fixer.fix(&mut coverage).unwrap();

    assert_eq!(
        coverage.file_coverages()[0].line_coverages(),
        &*expected_line_covs
    );
}

#[test]
fn derives() {
    let ws = WorkSpace::from_template("./tests/fix");
    let source_file = ws.path().join("derives.rs");

    let original_line_covs = line_coverages!(
        0 => 0,
        2 => 0,
        3 => 0,
        7 => 0,
        8 => 0,
        9 => 0,
        10 => 0,
        12 => 0,
        15 => 0,
        16 => 0,
        19 => 0,
        20 => 0,
        21 => 0,
        23 => 0,
        24 => 0,
        25 => 0,
        27 => 0,
        28 => 0,
        29 => 0,
        32 => 0,
        33 => 0,
        34 => 0,
        35 => 0,
        36 => 0,
        40 => 0,
        41 => 0,
        42 => 0,
    );

    let expected_line_covs = line_coverages!(
        7 => 0,
        8 => 0,
        9 => 0,
        10 => 0,
        19 => 0,
        20 => 0,
        23 => 0,
        24 => 0,
        27 => 0,
        28 => 0,
        40 => 0,
        41 => 0,
    );

    let mut coverage = PackageCoverage::new(vec![FileCoverage::new(
        &source_file,
        original_line_covs,
        vec![],
    )]);

    let fixer = CoverageFixer::new();
    fixer.fix(&mut coverage).unwrap();

    assert_eq!(
        coverage.file_coverages()[0].line_coverages(),
        &*expected_line_covs
    );
}

#[test]
fn comments() {
    let ws = WorkSpace::from_template("./tests/fix");
    let source_file = ws.path().join("comments.rs");

    let original_line_covs = line_coverages!(
        0 => 1,
        1 => 1,
        4 => 1,
        5 => 1,
        9 => 1,
        10 => 1,
        11 => 0,
        12 => 0,
        17 => 1,
        20 => 1,
        22 => 1,
    );

    let original_branch_covs = branch_coverages!(
        4 => true,
        4 => false,
        9 => true,
        9 => false,
        10 => true,
        10 => false,
        11 => false,
        11 => false,
        12 => false,
        12 => false,
        17 => true,
        17 => false,
        20 => true,
        20 => false,
    );

    let expected_line_covs = line_coverages!(
        0 => 1,
        4 => 1,
        5 => 1,
        20 => 1
    );

    let expected_branch_covs = branch_coverages!(
        9 => true,
        9 => false,
        10 => true,
        10 => false,
        11 => false,
        11 => false,
        12 => false,
        12 => false,
    );

    let mut coverage = PackageCoverage::new(vec![FileCoverage::new(
        &source_file,
        original_line_covs,
        original_branch_covs,
    )]);

    let fixer = CoverageFixer::new();
    fixer.fix(&mut coverage).unwrap();

    assert_eq!(
        coverage.file_coverages()[0].line_coverages(),
        &*expected_line_covs
    );

    assert_eq!(
        coverage.file_coverages()[0].branch_coverages(),
        &*expected_branch_covs
    );
}

#[test]
fn loops() {
    let ws = WorkSpace::from_template("./tests/fix");
    let source_file = ws.path().join("loops.rs");

    let original_line_covs = line_coverages!(
        0 => 1,
        1 => 11,
        2 => 10,
        6 => 1,
        7 => 1,
        10 => 1,
        13 => 11,
        14 => 10,
        21 => 0,
        22 => 0
    );

    let original_branch_covs = branch_coverages!(
        1 => true,
        1 => true,
        1 => false,
        13 => true,
        13 => true
    );

    let expected_line_covs = line_coverages!(
        0 => 1,
        1 => 11,
        2 => 10,
        6 => 1,
        7 => 1,
        10 => 1,
        13 => 11,
        14 => 10,
        21 => 0,
        22 => 0
    );

    let expected_branch_covs = branch_coverages!(
        1 => true,
        1 => true,
        13 => true,
        13 => true
    );

    let mut coverage = PackageCoverage::new(vec![FileCoverage::new(
        &source_file,
        original_line_covs,
        original_branch_covs,
    )]);

    let fixer = CoverageFixer::new();
    fixer.fix(&mut coverage).unwrap();

    assert_eq!(
        coverage.file_coverages()[0].line_coverages(),
        &*expected_line_covs
    );

    assert_eq!(
        coverage.file_coverages()[0].branch_coverages(),
        &*expected_branch_covs
    );
}
