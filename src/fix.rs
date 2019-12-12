use regex::Regex;
use std::fs;

use crate::common::{LineCoverage, PackageCoverage, SourceCode};

pub fn fix_coverage(data: &mut PackageCoverage) {
    let non_executable_lines = vec![
        Regex::new(r"^(?:\s*\}(?:\s*\))*(?:\s*;)?|\s*(?:\}\s*)?else(?:\s*\{)?)?\s*(?://.*)?$")
            .unwrap(),
        Regex::new(r"^\s*pub\s*struct\s*.*?\{\s*(?://.*)?$").unwrap(),
    ];

    for (file, covs) in data.iter_mut() {
        let content = fs::read_to_string(file).unwrap();
        let source = SourceCode::new(content);

        debug_assert!(
            source.total_lines() == covs.len(),
            "Internal error: line length did not match"
        );
        for (line, cov) in source.lines().zip(covs) {
            if *cov == LineCoverage::NotCovered {
                if non_executable_lines.iter().any(|r| r.is_match(line)) {
                    *cov = LineCoverage::NotExecutable;
                }
            }
        }
    }
}
