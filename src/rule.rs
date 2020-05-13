use proc_macro2::TokenTree;
use regex::Regex;
use std::fs;
use std::marker::PhantomData;
use std::path::Path;
use syn::visit::Visit;
use syn::{ExprForLoop, Fields, File, ItemEnum, ItemFn, ItemMod, ItemStruct, ItemUnion};

use crate::error::*;
use crate::{BranchCoverage, FileCoverage, LineCoverage};

pub struct SourceCode {
    pub content: String,
    pub ast: File,
}

impl SourceCode {
    pub fn new(filename: &Path) -> Result<SourceCode, Error> {
        let content = fs::read_to_string(filename)
            .chain_err(|| ErrorKind::SourceFileNotFound(filename.to_owned()))?;
        let ast =
            syn::parse_file(&content).chain_err(|| format!("Failed to parse {:?}", filename))?;
        Ok(SourceCode { content, ast })
    }
}

pub trait Rule {
    fn fix_file_coverage(&self, source: &SourceCode, file_cov: &mut FileCoverage);
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
    fn fix_file_coverage(&self, source: &SourceCode, file_cov: &mut FileCoverage) {
        for entry in PerLineIterator::new(&source.content, file_cov) {
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

pub struct TestRule;

impl TestRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for TestRule {
    fn fix_file_coverage(&self, source: &SourceCode, file_cov: &mut FileCoverage) {
        let mut inner = TestRuleInner { file_cov };
        inner.visit_file(&source.ast);
    }
}

struct TestRuleInner<'a> {
    file_cov: &'a mut FileCoverage,
}

impl<'a> TestRuleInner<'a> {
    fn ignore_range(&mut self, start: usize, end: usize) {
        for line_cov in self
            .file_cov
            .line_coverages
            .iter_mut()
            .skip_while(|e| e.line_number < start)
            .take_while(|e| e.line_number <= end)
        {
            line_cov.count = None;
        }

        for branch_cov in self
            .file_cov
            .branch_coverages
            .iter_mut()
            .skip_while(|e| e.line_number < start)
            .take_while(|e| e.line_number <= end)
        {
            branch_cov.taken = None;
        }
    }
}

impl<'ast, 'a> Visit<'ast> for TestRuleInner<'a> {
    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        let start = match item.attrs.get(0) {
            Some(attr) => attr.pound_token.spans[0].start().line,
            None => return,
        };
        let end = item.block.brace_token.span.end().line;

        for attr in item.attrs.iter() {
            if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "test" {
                self.ignore_range(start, end);
                return;
            }
        }

        syn::visit::visit_item_fn(self, item);
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        let span = match item.content {
            Some((ref brace, _)) => brace.span,
            None => return,
        };

        for attr in item.attrs.iter() {
            if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "cfg" {
                for token in attr.tokens.clone() {
                    if let TokenTree::Group(g) = token {
                        let token = match g.stream().into_iter().next() {
                            Some(t) => t,
                            None => continue,
                        };

                        if let TokenTree::Ident(ident) = token {
                            if ident == "test" {
                                self.ignore_range(span.start().line, span.end().line);
                                return;
                            }
                        }
                    }
                }
            }
        }

        syn::visit::visit_item_mod(self, item);
    }
}

pub struct LoopRule;

impl LoopRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for LoopRule {
    fn fix_file_coverage(&self, source: &SourceCode, file_cov: &mut FileCoverage) {
        let mut inner = LoopRuleInner {
            it: PerLineIterator::new(&source.content, file_cov),
            current_line: 0,
        };
        inner.visit_file(&source.ast);
    }
}

struct LoopRuleInner<'a, 'b> {
    it: PerLineIterator<'a, 'b>,
    current_line: usize,
}

impl<'ast, 'a, 'b> Visit<'ast> for LoopRuleInner<'a, 'b> {
    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        let line = expr.for_token.span.start().line;

        if let Some(entry) = self.it.nth(line - self.current_line - 1) {
            self.current_line = line;

            let should_be_fixed = entry
                .line_cov
                .map_or(false, |v| v.count.map_or(false, |c| c > 0));

            if should_be_fixed {
                for branch_cov in entry.branch_covs {
                    if branch_cov.taken == Some(false) {
                        branch_cov.taken = None;
                        break;
                    }
                }
            }
        }

        syn::visit::visit_expr_for_loop(self, expr);
    }
}

pub struct DeriveRule;

impl DeriveRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for DeriveRule {
    fn fix_file_coverage(&self, source: &SourceCode, file_cov: &mut FileCoverage) {
        let mut inner = DeriveLoopInner { file_cov };
        inner.visit_file(&source.ast);
    }
}

struct DeriveLoopInner<'a> {
    file_cov: &'a mut FileCoverage,
}

impl<'a> DeriveLoopInner<'a> {
    fn ignore_range(&mut self, start: usize, end: usize) {
        for line_cov in self
            .file_cov
            .line_coverages
            .iter_mut()
            .skip_while(|e| e.line_number < start)
            .take_while(|e| e.line_number <= end)
        {
            line_cov.count = None;
        }

        for branch_cov in self
            .file_cov
            .branch_coverages
            .iter_mut()
            .skip_while(|e| e.line_number < start)
            .take_while(|e| e.line_number <= end)
        {
            branch_cov.taken = None;
        }
    }
}

impl<'ast, 'a> Visit<'ast> for DeriveLoopInner<'a> {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let start = match item.attrs.get(0) {
            Some(attr) => attr.pound_token.spans[0].start().line,
            None => return,
        };
        let end = match item.fields {
            Fields::Named(ref f) => f.brace_token.span.end().line,
            Fields::Unnamed(ref f) => f.paren_token.span.end().line,
            Fields::Unit => item.ident.span().end().line,
        };

        for attr in item.attrs.iter() {
            if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "derive" {
                self.ignore_range(start, end);
                return;
            }
        }
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        let start = match item.attrs.get(0) {
            Some(attr) => attr.pound_token.spans[0].start().line,
            None => return,
        };
        let end = item.brace_token.span.end().line;

        for attr in item.attrs.iter() {
            if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "derive" {
                self.ignore_range(start, end);
                break;
            }
        }
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        let start = match item.attrs.get(0) {
            Some(attr) => attr.pound_token.spans[0].start().line,
            None => return,
        };
        let end = item.fields.brace_token.span.end().line;

        for attr in item.attrs.iter() {
            if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "derive" {
                self.ignore_range(start, end);
                break;
            }
        }
    }
}

pub struct CommentRule;

impl CommentRule {
    fn new() -> Self {
        Self
    }
}

impl Rule for CommentRule {
    fn fix_file_coverage(&self, source: &SourceCode, file_cov: &mut FileCoverage) {
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

        for mut entry in PerLineIterator::new(&source.content, file_cov) {
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
            line_number: 1,
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
        if self.line_number > self.lines.len() {
            return None;
        }

        unsafe {
            let line = self.lines.get_unchecked_mut(self.line_number - 1);

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

fn extract_marker(line: &str) -> Option<CommentMarker> {
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

// cov:begin-ignore
macro_rules! impl_default {
    ($name:ident) => {
        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

impl_default!(CloseBlockRule);
impl_default!(TestRule);
impl_default!(LoopRule);
impl_default!(DeriveRule);
impl_default!(CommentRule);
// cov:end-ignore

#[cfg(test)]
mod tests {
    #[test]
    fn extract_marker() {
        use super::CommentMarker::*;
        assert_eq!(super::extract_marker("ccov:ignore"), None);
        assert_eq!(super::extract_marker("cov:ignore-linee"), None);
        assert_eq!(super::extract_marker("cov:ignore--branch"), None);
        assert_eq!(super::extract_marker("cov:ignore"), Some(IgnoreBoth));
        assert_eq!(super::extract_marker("cov:ignore-line-begin"), None);
        assert_eq!(super::extract_marker("cov:ignore-end"), None);
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
