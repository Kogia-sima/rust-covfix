use std::fs;
use std::path::Path;

mod common;
use common::WorkSpace;

use rust_covfix::CoberturaParser;
use rust_covfix::{fix_coverage, CoverageReader, CoverageWriter};

fn assert_file_eq(p1: &Path, p2: &Path) {
    let c1 = fs::read_to_string(p1).unwrap_or_else(|_| panic!("Failed to open '{:?}'", p1));
    let c2 = fs::read_to_string(p2).unwrap_or_else(|_| panic!("Failed to open '{:?}'", p2));

    assert_eq!(c1, c2);
}

#[test]
fn test() {
    let ws = WorkSpace::from_template("high_complexity");
    let root = ws.path();

    let incorrect_file = root.join("incorrect").join("cobertura.xml");
    let correct_file = root.join("correct").join("cobertura.xml");

    let cf = CoberturaParser::new(ws.path());
    let mut coverage = cf.load_coverages(&incorrect_file);
    fix_coverage(&mut coverage);
    cf.save_coverages(&incorrect_file, &coverage);

    assert_file_eq(&incorrect_file, &correct_file);
}
