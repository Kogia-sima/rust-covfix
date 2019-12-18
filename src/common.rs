use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
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

    #[cfg_attr(not(feature = "coverage"), inline)]
    pub fn from_file(path: &Path) -> Self {
        let buffer = std::fs::read_to_string(path).unwrap();
        Self::new(buffer)
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
    pub fn get_line(&self, index: usize) -> Option<&str> {
        unsafe { self.lines.get(index).map(|&v| &*v) }
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
    pub fn lines<'a>(&'a self) -> Lines<'a> {
        Lines {
            inner: (&self.lines).into_iter(),
        }
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
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

    #[cfg_attr(not(feature = "coverage"), inline)]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.inner.next().map(|v| &**v) }
    }
}

unsafe impl Send for Lines<'_> {}
unsafe impl Sync for Lines<'_> {}

#[derive(Clone, Debug)]
pub struct LineCoverage {
    pub line_number: usize,
    pub count: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct BranchCoverage {
    pub line_number: Option<usize>,
    pub block_number: Option<usize>,
    pub branch_number: Option<usize>,
    pub taken: bool,
}

#[derive(Debug)]
pub struct FileCoverage {
    path: PathBuf,
    #[doc(hidden)]
    pub line_coverages: Vec<LineCoverage>,
    #[doc(hidden)]
    pub branch_coverages: Vec<BranchCoverage>,
}

impl FileCoverage {
    #[cfg_attr(not(feature = "coverage"), inline)]
    pub fn new<P: Into<PathBuf>>(
        path: P,
        line_coverages: Vec<LineCoverage>,
        branch_coverages: Vec<BranchCoverage>,
    ) -> Self {
        Self {
            path: path.into(),
            line_coverages,
            branch_coverages,
        }
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
    pub fn line_coverages(&self) -> &[LineCoverage] {
        &self.line_coverages
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
    pub fn branch_coverages(&self) -> &[BranchCoverage] {
        &self.branch_coverages
    }
}

#[derive(Debug)]
pub struct PackageCoverage {
    name: String,
    #[doc(hidden)]
    pub file_coverages: Vec<FileCoverage>,
}

impl PackageCoverage {
    pub fn new<T: Into<String>>(name: T, file_coverages: Vec<FileCoverage>) -> Self {
        Self {
            name: name.into(),
            file_coverages,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn file_coverages(&self) -> &[FileCoverage] {
        &self.file_coverages
    }
}

pub trait TotalCoverage {
    fn line_executed(&self) -> usize;
    fn line_total(&self) -> usize;
    fn branch_executed(&self) -> usize;
    fn branch_total(&self) -> usize;
}

impl TotalCoverage for FileCoverage {
    #[cfg_attr(not(feature = "coverage"), inline)]
    fn line_executed(&self) -> usize {
        self.line_coverages
            .iter()
            .filter(|&v| v.count.map_or(false, |v| v > 0))
            .count()
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
    fn line_total(&self) -> usize {
        self.line_coverages
            .iter()
            .filter(|&v| v.count.is_some())
            .count()
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
    fn branch_executed(&self) -> usize {
        self.branch_coverages.iter().filter(|&v| v.taken).count()
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
    fn branch_total(&self) -> usize {
        self.branch_coverages.len()
    }
}

impl TotalCoverage for PackageCoverage {
    #[cfg_attr(not(feature = "coverage"), inline)]
    fn line_executed(&self) -> usize {
        self.file_coverages
            .iter()
            .fold(0, |sum, a| sum + a.line_executed())
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
    fn line_total(&self) -> usize {
        self.file_coverages
            .iter()
            .fold(0, |sum, a| sum + a.line_total())
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
    fn branch_executed(&self) -> usize {
        self.file_coverages
            .iter()
            .fold(0, |sum, a| sum + a.branch_executed())
    }

    #[cfg_attr(not(feature = "coverage"), inline)]
    fn branch_total(&self) -> usize {
        self.file_coverages
            .iter()
            .fold(0, |sum, a| sum + a.branch_total())
    }
}

pub trait SupportedFile {
    fn is_supported(&self, path: &Path) -> bool;
}

pub trait CoverageReader {
    fn read<R: BufRead>(&self, reader: &mut R) -> PackageCoverage;

    fn read_from_file<P: AsRef<Path>>(&self, path: P) -> PackageCoverage {
        let f = fs::File::open(path).unwrap();
        let capacity = f.metadata().map(|m| m.len() as usize + 1).unwrap_or(0);
        let mut reader = BufReader::with_capacity(capacity, f);
        self.read(&mut reader)
    }
}

pub trait CoverageWriter {
    fn write<W: Write>(&self, data: &PackageCoverage, writer: &mut W);

    fn write_to_file<P: AsRef<Path>>(&self, data: &PackageCoverage, path: P) {
        let f = fs::File::create(path.as_ref()).unwrap();
        let capacity = f.metadata().map(|m| m.len() as usize + 1).unwrap_or(0);
        let mut writer = BufWriter::with_capacity(capacity, f);
        self.write(&data, &mut writer);
    }
}

#[cfg(test)]
mod tests {
    use super::SourceCode;

    #[test]
    fn source_code_tests() {
        let s = SourceCode::new("apple\nbanana");
        let mut it = s.lines().clone();
        assert_eq!(it.next(), Some("apple"));
        assert_eq!(it.next(), Some("banana"));
        assert_eq!(it.next(), None);
    }
}
