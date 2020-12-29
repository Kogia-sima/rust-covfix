use super::WorkSpace;
use pretty_assertions::assert_eq;

use rust_covfix::rule;
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
        1 => 1,
        2 => 1,
        3 => 1,
        4 => 1,
        5 => 1,
        6 => 0,
        7 => 0,
        8 => 0,
        10 => 0,
        11 => 0,
    );

    let expected_line_covs = line_coverages!(
        1 => 1,
        2 => 1,
        3 => 1,
        4 => 1,
        5 => 1,
        6 => 0,
        8 => 0,
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
        1 => 1,
        2 => 1,
        3 => 1,
        12 => 1,
        13 => 1,
        21 => 1,
        22 => 1,
        26 => 1,
        27 => 1,
        33 => 1,
        34 => 1,
        39 => 1,
        40 => 1,
    );

    let original_branch_covs = branch_coverages!(
        13 => true,
        13 => false,
        22 => true,
        22 => false,
        40 => true,
        40 => false,
    );

    let expected_line_covs = line_coverages!(
        1 => 1,
        2 => 1,
        21 => 1,
        22 => 1,
        26 => 1,
        27 => 1,
        39 => 1,
        40 => 1,
    );

    let expected_branch_covs = branch_coverages!(
        22 => true,
        40 => true,
        40 => false,
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
fn derives() {
    let ws = WorkSpace::from_template("./tests/fix");
    let source_file = ws.path().join("derives.rs");

    let original_line_covs = line_coverages!(
        1 => 0,
        3 => 0,
        4 => 0,
        8 => 0,
        9 => 0,
        10 => 0,
        11 => 0,
        13 => 0,
        16 => 0,
        17 => 0,
        20 => 0,
        21 => 0,
        22 => 0,
        24 => 0,
        25 => 0,
        26 => 0,
        28 => 0,
        29 => 0,
        30 => 0,
        33 => 0,
        34 => 0,
        35 => 0,
        36 => 0,
        37 => 0,
        41 => 0,
        42 => 0,
        43 => 0,
        46 => 0,
        47 => 0,
        48 => 0,
        49 => 0,
        52 => 0,
        53 => 0,
    );

    let original_branch_covs = branch_coverages!(
        17 => false,
    );

    let expected_line_covs = line_coverages!(
        8 => 0,
        9 => 0,
        10 => 0,
        11 => 0,
        20 => 0,
        21 => 0,
        24 => 0,
        25 => 0,
        28 => 0,
        29 => 0,
        41 => 0,
        42 => 0,
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

    assert_eq!(coverage.file_coverages()[0].branch_coverages(), &[]);
}

#[test]
fn comments() {
    let ws = WorkSpace::from_template("./tests/fix");
    let source_file = ws.path().join("comments.rs");

    let original_line_covs = line_coverages!(
        1 => 1,
        2 => 1,
        5 => 1,
        6 => 1,
        10 => 1,
        11 => 1,
        12 => 0,
        13 => 0,
        18 => 1,
        21 => 1,
        23 => 1,
    );

    let original_branch_covs = branch_coverages!(
        5 => true,
        5 => false,
        10 => true,
        10 => false,
        11 => true,
        11 => false,
        12 => false,
        12 => false,
        13 => false,
        13 => false,
        18 => true,
        18 => false,
        21 => true,
        21 => false,
    );

    let expected_line_covs = line_coverages!(
        1 => 1,
        5 => 1,
        6 => 1,
        21 => 1
    );

    let expected_branch_covs = branch_coverages!(
        10 => true,
        10 => false,
        11 => true,
        11 => false,
        12 => false,
        12 => false,
        13 => false,
        13 => false,
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
        1 => 1,
        2 => 11,
        3 => 10,
        7 => 1,
        8 => 1,
        11 => 1,
        14 => 11,
        15 => 10,
        22 => 0,
        23 => 0
    );

    let original_branch_covs = branch_coverages!(
        2 => true,
        2 => true,
        2 => false,
        14 => true,
        14 => true
    );

    let expected_line_covs = line_coverages!(
        1 => 1,
        2 => 11,
        3 => 10,
        7 => 1,
        8 => 1,
        11 => 1,
        14 => 11,
        15 => 10,
        22 => 0,
        23 => 0
    );

    let expected_branch_covs = branch_coverages!(
        2 => true,
        2 => true,
        14 => true,
        14 => true
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
fn unreachable() {
    let ws = WorkSpace::from_template("./tests/fix");
    let source_file = ws.path().join("unreachable.rs");

    let original_line_covs = line_coverages!(
        6 => 1,
        7 => 1,
        8 => 1,
        9 => 1,
        10 => 1,
        12 => 1,
        16 => 1,
        17 => 1,
        18 => 0,
        19 => 1,
        20 => 1,
        22 => 1,
    );

    let original_branch_covs = branch_coverages!(
        7 => true,
        7 => true,
        17 => true,
        17 => false,
    );

    let expected_line_covs = line_coverages!(
        6 => 1,
        7 => 1,
        8 => 1,
        10 => 1,
        16 => 1,
        17 => 1,
        20 => 1,
    );

    let expected_branch_covs = branch_coverages!(
        7 => true,
        7 => true,
        17 => true,
        17 => false,
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
fn assert() {
    let ws = WorkSpace::from_template("./tests/fix");
    let source_file = ws.path().join("assert.rs");

    let original_branch_covs = branch_coverages!(
        8 => true,
        8 => true,
        14 => true,
        14 => true,
        15 => true,
        15 => false,
        21 => true,
        21 => true,
        22 => false,
        22 => true,
        28 => true,
        28 => false,
        29 => false,
        29 => false,
    );

    let expected_branch_covs = branch_coverages!(
        8 => true,
        8 => true,
        14 => true,
        14 => true,
        15 => true,
        21 => true,
        21 => true,
        22 => true,
        28 => true,
        28 => false,
        29 => false,
    );

    let mut coverage = PackageCoverage::new(vec![FileCoverage::new(
        &source_file,
        Vec::new(),
        original_branch_covs,
    )]);

    let fixer = CoverageFixer::with_rules(vec![rule::from_str("assert").unwrap()]);
    fixer.fix(&mut coverage).unwrap();

    assert_eq!(
        coverage.file_coverages()[0].branch_coverages(),
        &*expected_branch_covs
    );
}
