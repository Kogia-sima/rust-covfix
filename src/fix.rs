use std::fs;

use crate::coverage::PackageCoverage;
use crate::error::*;
use crate::rule::{default_rules, Rule};

pub struct CoverageFixer {
    rules: Vec<Box<dyn Rule>>,
}

impl CoverageFixer {
    pub fn new() -> Self {
        Self {
            rules: default_rules(),
        }
    }

    /// fix coverage information
    pub fn fix(&self, data: &mut PackageCoverage) -> Result<(), Error> {
        for mut file_cov in &mut data.file_coverages {
            file_cov
                .line_coverages
                .sort_unstable_by_key(|v| v.line_number);
            file_cov
                .branch_coverages
                .sort_unstable_by_key(|v| v.line_number);

            let path = file_cov.path();
            let source = fs::read_to_string(path)
                .chain_err(|| format!("Failed to open source file: {:?}", path))?;

            for rule in &self.rules {
                rule.fix_file_coverage(&source, &mut file_cov);
            }
        }

        Ok(())
    }
}
