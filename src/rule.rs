use regex::Regex;
use std::marker::PhantomData;

use crate::{BranchCoverage, FileCoverage, LineCoverage};

struct CoverageEntry<'a, 'b> {
    line: &'a str,
    line_cov: Option<&'b mut LineCoverage>,
    branch_covs: &'b mut [BranchCoverage],
}

/// A coverage iterator over the lines of a source files.
#[derive(Clone)]
struct PerLineIterator<'a, 'b> {
    line_number: usize,
    lines: Vec<&'a str>,
    lp: *mut LineCoverage,
    lp_end: *mut LineCoverage,
    bp: *mut BranchCoverage,
    bp_end: *mut BranchCoverage,
    _borrow: PhantomData<&'b FileCoverage>,
}

impl<'a, 'b> PerLineIterator<'a, 'b> {
    fn new(source: &'a str, file_cov: &'b mut FileCoverage) -> PerLineIterator<'a, 'b> {
        let lp = file_cov.line_coverages.as_mut_ptr();
        let bp = file_cov.branch_coverages.as_mut_ptr();
        let lp_end = unsafe { lp.add(file_cov.line_coverages.len()) };
        let bp_end = unsafe { bp.add(file_cov.branch_coverages.len()) };

        Self {
            line_number: 0,
            lines: source.lines().collect(),
            lp,
            bp,
            lp_end,
            bp_end,
            _borrow: PhantomData,
        }
    }
}

impl<'a, 'b> Iterator for PerLineIterator<'a, 'b> {
    type Item = CoverageEntry<'a, 'b>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.line_number >= self.lines.len() {
            return None;
        }

        unsafe {
            let line = self.lines.get_unchecked_mut(self.line_number);

            // line coverage at current line
            let line_cov = if self.lp < self.lp_end && (*self.lp).line_number == self.line_number {
                let val = Some(&mut *self.lp);
                self.lp = self.lp.add(1);
                val
            } else {
                None
            };

            // branch coverages at current line
            let branch_covs = if self.bp < self.bp_end
                && (*self.bp).line_number.unwrap() == self.line_number
            {
                let start = self.bp;
                self.bp = self.bp.add(1);
                let mut count = 1;
                while self.bp < self.bp_end && (*self.bp).line_number.unwrap() == self.line_number {
                    self.bp = self.bp.add(1);
                    count += 1;
                }
                ::std::slice::from_raw_parts_mut(start, count)
            } else {
                &mut []
            };

            self.line_number += 1;

            Some(CoverageEntry {
                line,
                line_cov,
                branch_covs,
            })
        }
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn nth(&mut self, idx: usize) -> Option<Self::Item> {
        self.line_number += idx;
        self.next()
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn count(self) -> usize {
        self.len()
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.len();
        (exact, Some(exact))
    }
}

impl ExactSizeIterator for PerLineIterator<'_, '_> {
    #[cfg_attr(not(feature = "noinline"), inline)]
    fn len(&self) -> usize {
        self.lines.len() - self.line_number
    }
}

pub trait Rule {
    fn fix_file_coverage(&self, source: &str, file_cov: &mut FileCoverage);
}

pub struct CloseBlockRule {
    reg: Regex,
}

impl CloseBlockRule {
    pub fn new() -> Self {
        Self {
            reg: Regex::new(
                r"^(?:\s*\}(?:\s*\))*(?:\s*;)?|\s*(?:\}\s*)?else(?:\s*\{)?)?\s*(?://.*)?$",
            )
            .unwrap(),
        }
    }
}

impl Rule for CloseBlockRule {
    fn fix_file_coverage(&self, source: &str, file_cov: &mut FileCoverage) {
        for entry in PerLineIterator::new(source, file_cov) {
            if entry.line_cov.is_none() && entry.branch_covs.is_empty() {
                continue;
            }

            if self.reg.is_match(entry.line) {
                if let Some(line_cov) = entry.line_cov {
                    line_cov.line_number = std::usize::MAX;
                }

                entry
                    .branch_covs
                    .iter_mut()
                    .for_each(|v| v.line_number = Some(std::usize::MAX));
            }
        }

        file_cov
            .line_coverages
            .retain(|v| v.line_number != std::usize::MAX);
        file_cov
            .branch_coverages
            .retain(|v| v.line_number != Some(std::usize::MAX));
    }
}

pub struct TestRule {
    cfg_reg: Regex,
    mod_reg: Regex,
}

impl TestRule {
    pub fn new() -> Self {
        Self {
            cfg_reg: Regex::new(
                r"^\s*#\s*\[\s*cfg\((?:test)|(?:.*[ \t\(]test[,\)]))\s*\]\s*(?://.*)?$",
            )
            .unwrap(),
            mod_reg: Regex::new(r"^\s*(?:pub\s+)?mod\s+tests?\s*\{").unwrap(),
        }
    }
}

impl Rule for TestRule {
    fn fix_file_coverage(&self, source: &str, file_cov: &mut FileCoverage) {
        fn ignore_coverages(entry: &mut CoverageEntry) {
            if let Some(&mut ref mut line_cov) = entry.line_cov {
                line_cov.line_number = std::usize::MAX;
            }

            entry
                .branch_covs
                .iter_mut()
                .for_each(|v| v.line_number = Some(std::usize::MAX));
        }

        let mut cfg_found = false;
        let mut inside_test = false;

        for mut entry in PerLineIterator::new(source, file_cov) {
            if inside_test {
                ignore_coverages(&mut entry);

                let line = entry.line.trim_start();
                if line.bytes().next() == Some(b'}') {
                    inside_test = false;
                }
            }
            if !cfg_found {
                if self.cfg_reg.is_match(entry.line) {
                    cfg_found = true;
                }
            } else {
                let line = entry.line.trim_start();
                if let Some(b) = line.bytes().next() {
                    if b != b'#' && b != b'/' {
                        cfg_found = false;
                        continue;
                    }
                }
                if self.mod_reg.is_match(entry.line) {
                    inside_test = true;
                    cfg_found = false;
                }
            }
        }

        file_cov
            .line_coverages
            .retain(|v| v.line_number != std::usize::MAX);
        file_cov
            .branch_coverages
            .retain(|v| v.line_number != Some(std::usize::MAX));
    }
}

pub struct LoopRule {
    loop_reg: Regex,
}

impl LoopRule {
    pub fn new() -> Self {
        Self {
            loop_reg: Regex::new(r"^\s*for\s*.*\{\s*(?://.*)?$").unwrap(),
        }
    }
}

impl Rule for LoopRule {
    fn fix_file_coverage(&self, source: &str, file_cov: &mut FileCoverage) {
        for entry in PerLineIterator::new(source, file_cov) {
            if entry.branch_covs.is_empty() {
                continue;
            }

            if self.loop_reg.is_match(entry.line) {
                for branch_cov in entry.branch_covs {
                    if !branch_cov.taken {
                        branch_cov.line_number = Some(std::usize::MAX);
                        break;
                    }
                }
            }
        }

        file_cov
            .branch_coverages
            .retain(|v| v.line_number != Some(std::usize::MAX));
    }
}

/*
pub struct DeriveRule {
    cfg_reg: Regex,
    decl_reg: Regex
}

impl DeriveRule {
    pub fn new() -> Self {
        Self {
            cfg_reg: Regex::new(r"^\s*#\s*\[\s*derive\(.*\)\s*\]").unwrap(),
            decl_reg: Regex::new(r"(:?pub\s*)?(?:struct)|(?:enum)|(?:union)\s*\w+").unwrap()
        }
    }

    pub fn fix_file_coverage(&self, source: &str, file_cov: &mut FileCoverage) {
        for entry in PerLineIterator::new(source, file_cov) {

        }
    }
}
*/
