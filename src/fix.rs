use crate::coverage::{PackageCoverage, TotalCoverage};
use crate::error::*;
use crate::rule::{default_rules, Rule, SourceCode};

/// Fix coverage information based on source code
///
/// You MUST fix coverage information using this struct because
/// Rules require coverage informations to be stored in correct format.
/// This struct checks the information format and modify it if it is invalid.
pub struct CoverageFixer {
    rules: Vec<Box<dyn Rule>>,
}

impl CoverageFixer {
    #[cfg_attr(feature = "noinline", inline(never))]
    pub fn new() -> Self {
        Self {
            rules: default_rules(),
        }
    }

    #[cfg_attr(feature = "noinline", inline(never))]
    pub fn with_rules<I: Into<Vec<Box<dyn Rule>>>>(rules: I) -> Self {
        Self {
            rules: rules.into(),
        }
    }

    /// fix coverage information
    pub fn fix(&self, data: &mut PackageCoverage) -> Result<(), Error> {
        if self.rules.is_empty() {
            debugln!("Skipping fix because rules are empty");
        }

        let old = CoverageSummary::new(data);

        debugln!("Fixing package coverage");
        for file_cov in &mut data.file_coverages {
            file_cov.line_coverages.sort_by_key(|v| v.line_number);
            file_cov.branch_coverages.sort_by_key(|v| v.line_number);

            let path = file_cov.path();
            debugln!("Processing file {:?}", path);

            let source = SourceCode::new(path)?;

            for rule in self.rules.iter() {
                rule.fix_file_coverage(&source, file_cov);
            }

            file_cov.line_coverages.retain(|v| v.count.is_some());
            file_cov.branch_coverages.retain(|v| v.taken.is_some());
        }

        let new = CoverageSummary::new(data);

        infoln!("Coverages are fixed successfully!");

        infoln!(
            "  line:   {:.2}% ({} of {} lines)    => {:.2}% ({} of {} lines)",
            old.line_percent(),
            old.line_executed,
            old.line_total,
            new.line_percent(),
            new.line_executed,
            new.line_total,
        );

        infoln!(
            "  branch: {:.2}% ({} of {} branches) => {:.2}% ({} of {} branches)\n",
            old.branch_percent(),
            old.branch_executed,
            old.branch_total,
            new.branch_percent(),
            new.branch_executed,
            new.branch_total,
        );

        Ok(())
    }
}

impl Default for CoverageFixer {
    fn default() -> Self {
        Self::new()
    }
}

struct CoverageSummary {
    line_executed: usize,
    line_total: usize,
    branch_executed: usize,
    branch_total: usize,
}

impl CoverageSummary {
    #[cfg_attr(feature = "noinline", inline(never))]
    fn new(data: &PackageCoverage) -> Self {
        Self {
            line_executed: data.line_executed(),
            line_total: data.line_total(),
            branch_executed: data.branch_executed(),
            branch_total: data.branch_total(),
        }
    }

    #[cfg_attr(feature = "noinline", inline(never))]
    fn line_percent(&self) -> f64 {
        (self.line_executed as f64) / (self.line_total as f64) * 100.0
    }

    #[cfg_attr(feature = "noinline", inline(never))]
    fn branch_percent(&self) -> f64 {
        (self.branch_executed as f64) / (self.branch_total as f64) * 100.0
    }
}
