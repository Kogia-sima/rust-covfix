use regex::Regex;
use std::marker::PhantomData;

use crate::error::*;
use crate::{BranchCoverage, FileCoverage, LineCoverage};

pub trait Rule {
    fn fix_file_coverage(&self, source: &str, file_cov: &mut FileCoverage);
}

pub struct CloseBlockRule {
    reg: Regex,
}

impl CloseBlockRule {
    #[cfg_attr(not(feature = "noinline"), inline)]
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
                    line_cov.count = None;
                }

                entry.branch_covs.iter_mut().for_each(|v| v.taken = None);
            }
        }
    }
}

pub struct TestRule {
    cfg_reg: Regex,
    mod_reg: Regex,
}

impl TestRule {
    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn new() -> Self {
        Self {
            cfg_reg: Regex::new(
                r"^\s*#\s*\[\s*cfg\((?:test)|(?:.*[ \t\(]test[,\)])\)\s*\]\s*(?://.*)?$",
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
                line_cov.count = None;
            }

            entry.branch_covs.iter_mut().for_each(|v| v.taken = None);
        }

        let mut cfg_found = false;
        let mut inside_test = false;

        for mut entry in PerLineIterator::new(source, file_cov) {
            if inside_test {
                ignore_coverages(&mut entry);

                if entry.line.bytes().next() == Some(b'}') {
                    inside_test = false;
                }
            } else if !cfg_found {
                if self.cfg_reg.is_match(entry.line) {
                    cfg_found = true;
                }
            } else {
                if self.mod_reg.is_match(entry.line) {
                    inside_test = true;
                    cfg_found = false;
                    continue;
                }

                let line = entry.line.trim_start();
                if let Some(b) = line.bytes().next() {
                    if b != b'#' && b != b'/' {
                        cfg_found = false;
                    }
                }
            }
        }
    }
}

pub struct LoopRule {
    loop_reg: Regex,
}

impl LoopRule {
    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn new() -> Self {
        Self {
            loop_reg: Regex::new(r"^\s*for\s*.*\{\s*(?://.*)?$").unwrap(),
        }
    }
}

impl Rule for LoopRule {
    fn fix_file_coverage(&self, source: &str, file_cov: &mut FileCoverage) {
        if file_cov.branch_coverages.is_empty() {
            debugln!(
                "Skipping LoopRule because the file coverage does not contain any branch coverage."
            );
            return;
        }

        for entry in PerLineIterator::new(source, file_cov) {
            if entry.branch_covs.is_empty() {
                continue;
            }

            let should_be_fixed = entry
                .line_cov
                .map_or(false, |v| v.count.map_or(false, |c| c > 0));

            if should_be_fixed && self.loop_reg.is_match(entry.line) {
                for branch_cov in entry.branch_covs {
                    if !branch_cov.taken.unwrap_or(false) {
                        branch_cov.taken = None;
                        break;
                    }
                }
            }
        }
    }
}

pub struct DeriveRule {
    cfg_reg: Regex,
    decl_reg: Regex,
}

impl DeriveRule {
    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn new() -> Self {
        Self {
            cfg_reg: Regex::new(r"^\s*#\s*\[\s*derive\(.*\)\s*\]").unwrap(),
            decl_reg: Regex::new(r"^\s*(:?pub\s*)?(?:struct)|(?:enum)|(?:union)\s*\w+").unwrap(),
        }
    }
}

impl Rule for DeriveRule {
    fn fix_file_coverage(&self, source: &str, file_cov: &mut FileCoverage) {
        fn ignore_coverages(entry: &mut CoverageEntry) {
            if let Some(&mut ref mut line_cov) = entry.line_cov {
                line_cov.count = None;
            }

            entry.branch_covs.iter_mut().for_each(|v| v.taken = None);
        }

        let mut cfg_found = false;
        let mut inside_derive = false;

        for mut entry in PerLineIterator::new(source, file_cov) {
            if inside_derive {
                ignore_coverages(&mut entry);

                let line = trim_comments(entry.line);
                if line.trim_start().bytes().next() == Some(b'#') {
                    // ignore cfg
                    continue;
                }

                if line.bytes().find(|&v| v == b'}').is_some() {
                    inside_derive = false;
                }
            } else if cfg_found {
                if self.decl_reg.is_match(entry.line) {
                    ignore_coverages(&mut entry);

                    let line = trim_comments(entry.line);
                    if line.bytes().find(|&v| v == b';' || v == b'}').is_none() {
                        inside_derive = true;
                        cfg_found = false;
                    }
                } else {
                    let line = entry.line.trim_start();
                    if let Some(b) = line.bytes().next() {
                        if b != b'#' && b != b'/' {
                            cfg_found = false;
                        }
                    }

                    if cfg_found {
                        ignore_coverages(&mut entry);
                    }
                }
            } else {
                if self.cfg_reg.is_match(entry.line) {
                    ignore_coverages(&mut entry);
                    cfg_found = true;
                }
            }
        }
    }
}

pub struct CommentRule;

impl CommentRule {
    #[cfg_attr(not(feature = "noinline"), inline)]
    fn new() -> Self {
        Self
    }
}

impl Rule for CommentRule {
    fn fix_file_coverage(&self, source: &str, file_cov: &mut FileCoverage) {
        fn ignore_line(entry: &mut CoverageEntry) {
            if let Some(&mut ref mut line_cov) = entry.line_cov {
                line_cov.count = None;
            }
        }

        fn ignore_branch(entry: &mut CoverageEntry) {
            entry.branch_covs.iter_mut().for_each(|v| v.taken = None);
        }

        fn ignore_both(entry: &mut CoverageEntry) {
            ignore_line(entry);
            ignore_branch(entry);
        }

        let mut inside_ignore_line = false;
        let mut inside_ignore_branch = false;
        let mut inside_ignore_both = false;

        for mut entry in PerLineIterator::new(source, file_cov) {
            use CommentMarker::*;

            let marker = extract_marker(entry.line);

            if inside_ignore_line {
                ignore_line(&mut entry);

                if marker == Some(EndIgnoreLine) {
                    inside_ignore_line = false;
                }

                continue;
            }

            if inside_ignore_branch {
                ignore_branch(&mut entry);

                if marker == Some(EndIgnoreBranch) {
                    inside_ignore_branch = false;
                }

                continue;
            }

            if inside_ignore_both {
                ignore_both(&mut entry);

                if marker == Some(EndIgnoreBoth) {
                    inside_ignore_both = false;
                }

                continue;
            }

            match marker {
                Some(IgnoreLine) => ignore_line(&mut entry),
                Some(IgnoreBranch) => ignore_branch(&mut entry),
                Some(IgnoreBoth) => ignore_both(&mut entry),
                Some(BeginIgnoreLine) => {
                    ignore_line(&mut entry);
                    inside_ignore_line = true;
                }
                Some(BeginIgnoreBranch) => {
                    ignore_branch(&mut entry);
                    inside_ignore_branch = true;
                }
                Some(BeginIgnoreBoth) => {
                    ignore_both(&mut entry);
                    inside_ignore_both = true;
                }
                _ => {}
            }
        }
    }
}

#[cfg_attr(not(feature = "noinline"), inline)]
pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(CloseBlockRule::new()),
        Box::new(TestRule::new()),
        Box::new(LoopRule::new()),
        Box::new(DeriveRule::new()),
        Box::new(CommentRule::new()),
    ]
}

pub fn from_str(s: &str) -> Result<Box<dyn Rule>, Error> {
    if s == "close" {
        return Ok(Box::new(CloseBlockRule::new()));
    }
    if s == "test" {
        return Ok(Box::new(TestRule::new()));
    }
    if s == "loop" {
        return Ok(Box::new(LoopRule::new()));
    }
    if s == "derive" {
        return Ok(Box::new(DeriveRule::new()));
    }
    if s == "comment" {
        return Ok(Box::new(CommentRule::new()));
    }

    Err(ErrorKind::InvalidRuleName(s.to_owned()).into())
}

// ---------- Utilities ----------

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
            let branch_covs = if self.bp < self.bp_end && (*self.bp).line_number == self.line_number
            {
                let start = self.bp;
                self.bp = self.bp.add(1);
                let mut count = 1;
                while self.bp < self.bp_end && (*self.bp).line_number == self.line_number {
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
}

fn trim_comments(s: &str) -> &str {
    let mut inside_quote = false;
    let mut slash_pos = s.len();

    for (i, b) in s.bytes().enumerate() {
        if inside_quote {
            if b == b'"' {
                inside_quote = false;
            }
        } else if b == b'"' {
            inside_quote = true;
        } else if b == b'/' {
            if slash_pos + 1 == i {
                return &s[0..slash_pos];
            } else {
                slash_pos = i;
            }
        }
    }

    s
}

#[derive(Debug, PartialEq)]
enum CommentMarker {
    IgnoreLine,
    IgnoreBranch,
    IgnoreBoth,
    BeginIgnoreLine,
    BeginIgnoreBranch,
    BeginIgnoreBoth,
    EndIgnoreLine,
    EndIgnoreBranch,
    EndIgnoreBoth,
}

fn extract_marker<'a>(line: &'a str) -> Option<CommentMarker> {
    fn is_character(byte: u8) -> bool {
        (0x41 <= byte && byte <= 0x5a)
            || (0x61 <= byte && byte <= 0x7a)
            || byte == b'_'
            || byte == b'-'
    }

    let bytes = line.as_bytes();
    let imax = bytes.len().saturating_sub(9);

    for i in 0..imax {
        unsafe {
            if !bytes.get_unchecked(i..).starts_with(b"cov:") {
                continue;
            }

            if i != 0 && !b" \t".contains(bytes.get_unchecked(i - 1)) {
                continue;
            }

            let mut pos = i + 4;
            while pos < bytes.len() && b" \t".contains(bytes.get_unchecked(pos)) {
                pos += 1;
            }

            let mut end_pos = pos + 1;
            while end_pos < bytes.len() && is_character(*bytes.get_unchecked(end_pos)) {
                end_pos += 1;
            }

            let key = std::str::from_utf8_unchecked(bytes.get_unchecked(pos..end_pos));

            return parse_marker(key);
        }
    }

    None
}

fn parse_marker(key: &str) -> Option<CommentMarker> {
    use CommentMarker::*;

    let mut splits = key.split(|v| v == '-' || v == '_');
    let mut segments = [""; 3];

    segments[0] = splits.next().unwrap_or("");
    segments[1] = splits.next().unwrap_or("");
    segments[2] = splits.next().unwrap_or("");

    match segments {
        ["ignore", "line", ""] => Some(IgnoreLine),
        ["ignore", "branch", ""] => Some(IgnoreBranch),
        ["ignore", "", ""] => Some(IgnoreBoth),
        ["begin", "ignore", "line"] => Some(BeginIgnoreLine),
        ["begin", "ignore", "branch"] => Some(BeginIgnoreBranch),
        ["begin", "ignore", ""] => Some(BeginIgnoreBoth),
        ["end", "ignore", "line"] => Some(EndIgnoreLine),
        ["end", "ignore", "branch"] => Some(EndIgnoreBranch),
        ["end", "ignore", ""] => Some(EndIgnoreBoth),
        _ => {
            warnln!("Warning: Invalid marker detected: {:?}", key);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn trim_comments() {
        assert_eq!(
            super::trim_comments("line: \"// comment\""),
            "line: \"// comment\""
        );
        assert_eq!(super::trim_comments("1 / 2"), "1 / 2");
        assert_eq!(super::trim_comments("1 // 2"), "1 ");
        assert_eq!(super::trim_comments("1 // 2 // 3"), "1 ");
        assert_eq!(super::trim_comments("1 + 2 // comment"), "1 + 2 ");
    }

    #[test]
    fn extract_marker() {
        use super::CommentMarker::*;
        assert_eq!(super::extract_marker("ccov:ignore"), None);
        assert_eq!(super::extract_marker("cov:ignore-linee"), None);
        assert_eq!(super::extract_marker("cov:ignore--branch"), None);
        assert_eq!(super::extract_marker("cov:ignore"), Some(IgnoreBoth));
        assert_eq!(
            super::extract_marker("//\tcov:ignore-line"),
            Some(IgnoreLine)
        );
        assert_eq!(
            super::extract_marker("// cov:ignore_branch"),
            Some(IgnoreBranch)
        );
        assert_eq!(
            super::extract_marker("cov:ignore-branch"),
            Some(IgnoreBranch)
        );
        assert_eq!(
            super::extract_marker("cov:begin-ignore"),
            Some(BeginIgnoreBoth)
        );
        assert_eq!(
            super::extract_marker("cov: begin-ignore-line"),
            Some(BeginIgnoreLine)
        );
        assert_eq!(
            super::extract_marker("cov: \tbegin-ignore-branch"),
            Some(BeginIgnoreBranch)
        );
        assert_eq!(
            super::extract_marker("cov:\t end_ignore-line"),
            Some(EndIgnoreLine)
        );
        assert_eq!(
            super::extract_marker("cov:end-ignore_branch\t"),
            Some(EndIgnoreBranch)
        );
        assert_eq!(
            super::extract_marker("cov:end-ignore "),
            Some(EndIgnoreBoth)
        );
    }

    #[test]
    fn from_str() {
        assert!(super::from_str("close").is_ok());
        assert!(super::from_str("test").is_ok());
        assert!(super::from_str("loop").is_ok());
        assert!(super::from_str("derive").is_ok());
        assert!(super::from_str("comment").is_ok());
        assert!(super::from_str("").is_err());
        assert!(super::from_str("derives").is_err());
        assert!(super::from_str("forloop").is_err());
    }
}
