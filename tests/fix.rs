use kcov_coverage_fix::{fix_coverage, LineCoverage, PackageCoverage};

mod common;
use common::WorkSpace;

#[test]
fn closing_brackets() {
    let ws = WorkSpace::from_template("fix");
    let p = ws.path().join("closing_brackets.rs");

    let fc = vec![LineCoverage::NotCovered; 5];

    let mut pc = PackageCoverage::new();
    pc.insert(p.clone(), fc);

    fix_coverage(&mut pc);

    assert_eq!(
        pc.get(&p),
        Some(&vec![
            LineCoverage::NotCovered,
            LineCoverage::NotCovered,
            LineCoverage::NotExecutable,
            LineCoverage::NotCovered,
            LineCoverage::NotExecutable,
        ])
    );
}

#[test]
fn struct_declaration() {
    let ws = WorkSpace::from_template("fix");
    let p = ws.path().join("struct_declaration.rs");

    let fc = vec![LineCoverage::NotCovered; 3];

    let mut pc = PackageCoverage::new();
    pc.insert(p.clone(), fc);

    fix_coverage(&mut pc);

    assert_eq!(
        pc.get(&p),
        Some(&vec![
            LineCoverage::NotExecutable,
            LineCoverage::NotCovered,
            LineCoverage::NotExecutable,
        ])
    );
}
