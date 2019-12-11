use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::slice;

pub struct SourceCode {
    // WARNING! Do not edit buffer after the object is initialized
    #[allow(dead_code)]
    buffer: String,
    lines: Vec<*const str>,
}

impl SourceCode {
    pub fn new<T: Into<String>>(buffer: T) -> Self {
        let buffer = buffer.into();
        let lines: Vec<_> = buffer.lines().map(|v| v as *const str).collect();
        SourceCode { buffer, lines }
    }

    #[inline]
    pub fn lines<'a>(&'a self) -> Lines<'a> {
        Lines {
            inner: (&self.lines).into_iter(),
        }
    }

    #[inline]
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }
}

unsafe impl Send for SourceCode {}
unsafe impl Sync for SourceCode {}

#[derive(Clone)]
pub struct Lines<'a> {
    inner: slice::Iter<'a, *const str>,
}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.inner.next().map(|v| &**v) }
    }
}

unsafe impl Send for Lines<'_> {}
unsafe impl Sync for Lines<'_> {}

pub trait TotalCoverage {
    fn line_rate(&self) -> f32 {
        1.0
    }
    fn branch_rate(&self) -> f32 {
        1.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineCoverage {
    NotCovered = 0,
    Covered = 1,
    NotExecutable = 2,
}

impl TotalCoverage for LineCoverage {
    #[inline]
    fn line_rate(&self) -> f32 {
        if *self == Self::Covered {
            1.0
        } else {
            0.0
        }
    }
}

impl TotalCoverage for FileCoverage {
    fn line_rate(&self) -> f32 {
        let mut hits = 0u32;
        let mut total = 0u32;

        for cov in self {
            match cov {
                LineCoverage::Covered => {
                    total += 1;
                    hits += 1;
                }
                LineCoverage::NotCovered => {
                    total += 1;
                }
                LineCoverage::NotExecutable => {}
            };
        }

        (hits as f32) / (total as f32)
    }
}

pub trait CoverageReader {
    fn load_coverages(&self, path: &Path) -> PackageCoverage;
}

pub trait CoverageWriter {
    fn save_coverages(&self, path: &Path, data: &PackageCoverage);
}

pub type FileCoverage = Vec<LineCoverage>;
pub type PackageCoverage = HashMap<PathBuf, FileCoverage>;

#[cfg(test)]
mod tests {
    use super::{LineCoverage, SourceCode, TotalCoverage};

    #[test]
    fn source_code_tests() {
        let s = SourceCode::new("apple\nbanana");
        let mut it = s.lines().clone();
        assert_eq!(it.next(), Some("apple"));
        assert_eq!(it.next(), Some("banana"));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn total_coverage() {
        assert_eq!(LineCoverage::NotExecutable.line_rate(), 0.0);
        assert_eq!(LineCoverage::NotExecutable.branch_rate(), 1.0);
        assert_eq!(LineCoverage::NotCovered.line_rate(), 0.0);
        assert_eq!(LineCoverage::Covered.line_rate(), 1.0);
        assert_eq!(
            vec![
                LineCoverage::NotCovered,
                LineCoverage::NotCovered,
                LineCoverage::NotCovered
            ]
            .line_rate(),
            0.0
        );
        assert_eq!(
            vec![
                LineCoverage::NotCovered,
                LineCoverage::Covered,
                LineCoverage::NotCovered
            ]
            .line_rate(),
            1.0 / 3.0
        );
    }
}
