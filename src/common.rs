use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::fs;
use std::iter::FusedIterator;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::slice;
use std::str::FromStr;
use std::string::ToString;

pub struct SourceCode {
    // WARNING! Do not edit buffer after the object is initialized
    buffer: String,
    lines: Vec<*const str>
}

impl SourceCode {
    pub fn new<T: Into<String>>(buffer: T) -> Self {
        let buffer = buffer.into();
        let lines: Vec<_> = buffer.lines().map(|v| v as *const str).collect();
        SourceCode { buffer, lines }
    }

    #[inline]
    #[doc(hidden)]
    pub fn from_raw(buffer: String, lines: Vec<*const str>) -> Self {
        SourceCode { buffer, lines }
    }

    #[inline]
    pub fn lines<'a>(&'a self) -> Lines<'a> {
        Lines {
            inner: (&self.lines).into_iter()
        }
    }

    #[inline]
    pub fn get_line<'a>(&'a self, idx: usize) -> Option<&'a str> {
        unsafe { self.lines.get(idx).map(|v| &**v) }
    }

    #[inline]
    pub fn get_line_unchecked<'a>(&'a self, idx: usize) -> &'a str {
        unsafe { &**self.lines.get_unchecked(idx) }
    }

    #[inline]
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }
}

impl Clone for SourceCode {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.buffer.clone())
    }
}

impl PartialEq<SourceCode> for SourceCode {
    #[inline]
    fn eq(&self, rhs: &SourceCode) -> bool {
        self.buffer == rhs.buffer
    }
}

impl Eq for SourceCode {}

impl Hash for SourceCode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&self.buffer).hash(state)
    }
}

impl fmt::Display for SourceCode {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&*self.buffer, f)
    }
}

impl fmt::Debug for SourceCode {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*self.buffer, f)
    }
}

impl From<String> for SourceCode {
    #[inline]
    fn from(buffer: String) -> Self {
        Self::new(buffer)
    }
}

impl Into<String> for SourceCode {
    #[inline]
    fn into(self) -> String {
        self.buffer
    }
}

impl AsRef<str> for SourceCode {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.buffer
    }
}

impl FromStr for SourceCode {
    type Err = Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Infallible> {
        Ok(Self::new(s))
    }
}

unsafe impl Send for SourceCode {}
unsafe impl Sync for SourceCode {}

impl<'a> IntoIterator for &'a SourceCode {
    type IntoIter = Lines<'a>;
    type Item = &'a str;

    fn into_iter(self) -> Self::IntoIter {
        Lines {
            inner: (&self.lines).into_iter()
        }
    }
}

#[derive(Clone)]
pub struct Lines<'a> {
    inner: slice::Iter<'a, *const str>
}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.inner.next().map(|v| &**v) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        unsafe { self.inner.nth(n).map(|v| &**v) }
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        unsafe { self.inner.last().map(|v| &**v) }
    }
}

impl DoubleEndedIterator for Lines<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe { self.inner.next_back().map(|v| &**v) }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        unsafe { self.inner.nth_back(n).map(|v| &**v) }
    }
}

impl ExactSizeIterator for Lines<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl FusedIterator for Lines<'_> {}

unsafe impl Send for Lines<'_> {}
unsafe impl Sync for Lines<'_> {}

#[derive(Clone, Debug)]
pub struct SourceFile {
    path: PathBuf,
    lines: Vec<String>,
}

impl SourceFile {
    pub fn new<T: Into<PathBuf>>(path: T) -> SourceFile {
        let path = path.into();
        let buffer = fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!("Failed to open {}", path.display());
        });
        let lines = buffer.lines().map(|v| v.to_string()).collect();

        SourceFile { path, lines }
    }

    /// get full path of source file
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// get reference to lines
    #[inline]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// get the source of n-th line, where line index starts from zero.
    #[inline]
    pub fn get_line(&self, index: usize) -> Option<&str> {
        match self.lines.get(index) {
            Some(ref s) => Some(&*s),
            None => None
        }
    }

    /// return the total number of lines in the source file
    #[inline]
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }
}

impl PartialEq for SourceFile {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for SourceFile {}

impl Hash for SourceFile {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state)
    }
}

pub trait TotalCoverage {
    fn line_rate(&self) -> f32 { 1.0 }
    fn branch_rate(&self) -> f32 { 1.0 }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineCoverage {
    NotCovered = 0,
    Covered = 1,
    NotExecutable = 2,
}

impl Default for LineCoverage {
    fn default() -> Self {
        Self::NotExecutable
    }
}

impl TotalCoverage for LineCoverage {
    #[inline]
    fn line_rate(&self) -> f32 {
        if *self == Self::Covered { 1.0 } else { 0.0 }
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
                },
                LineCoverage::NotCovered => {
                    total += 1;
                },
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
    use super::SourceCode;

    #[test]
    fn test1() {
        let s = SourceCode::new(
            "apple\nbanana"
        );
        let mut it = s.into_iter();
        assert_eq!(it.next(), Some("apple"));
        assert_eq!(it.next(), Some("banana"));
        assert_eq!(it.next(), None);
    }
}
