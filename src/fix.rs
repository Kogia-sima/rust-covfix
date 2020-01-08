use std::fs;

use crate::coverage::PackageCoverage;
use crate::error::*;
use crate::rule::{default_rules, Rule};

/// Fix coverage information based on source code
///
/// You MUST fix coverage information using this struct because
/// Rules require coverage informations to be stored in correct format.
/// This struct checks the information format and modify it if it is invalid.
pub struct CoverageFixer {
    rules: Vec<Box<dyn Rule>>,
}

impl CoverageFixer {
    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn new() -> Self {
        Self {
            rules: default_rules(),
        }
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn with_rules<I: Into<Vec<Box<dyn Rule>>>>(rules: I) -> Self {
        Self {
            rules: rules.into(),
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

            file_cov.line_coverages.retain(|v| v.count.is_some());
            file_cov.branch_coverages.retain(|v| v.taken.is_some());
        }

        Ok(())
    }
}
