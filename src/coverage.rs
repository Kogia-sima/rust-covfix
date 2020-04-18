use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::error::*;

/// Coverage information for a single line
#[derive(Clone, Debug, PartialEq)]
pub struct LineCoverage {
    /// 1-indexed line in the source file
    pub line_number: usize,
    /// execution count of line. `None` means this line is not executable.
    /// `None` value is used when the fixer detects non-executable line.
    pub count: Option<u32>,
}

/// Coverage information for a single branch
#[derive(Clone, Debug, PartialEq)]
pub struct BranchCoverage {
    /// 1-indexed line in the source file
    pub line_number: usize,
    /// block id which contains this branch
    pub block_number: Option<usize>,
    /// whether this branch was executed.
    /// `None` value is used when the fixer detects non-executable branch.
    pub taken: Option<bool>,
}

/// Coverage information for a single file
///
/// `FileCoverage` holds coverage information for lines and branches in the source file.
#[derive(Debug, PartialEq)]
pub struct FileCoverage {
    path: PathBuf,
    #[doc(hidden)]
    pub line_coverages: Vec<LineCoverage>,
    #[doc(hidden)]
    pub branch_coverages: Vec<BranchCoverage>,
}

impl FileCoverage {
    #[cfg_attr(not(feature = "noinline"), inline)]
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

    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn line_coverages(&self) -> &[LineCoverage] {
        &self.line_coverages
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn branch_coverages(&self) -> &[BranchCoverage] {
        &self.branch_coverages
    }
}

/// Coverage information for package
#[derive(Debug, PartialEq)]
pub struct PackageCoverage {
    name: String,
    #[doc(hidden)]
    pub file_coverages: Vec<FileCoverage>,
}

impl PackageCoverage {
    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn new(file_coverages: Vec<FileCoverage>) -> Self {
        Self::with_test_name("", file_coverages)
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn with_test_name<T: Into<String>>(name: T, file_coverages: Vec<FileCoverage>) -> Self {
        Self {
            name: name.into(),
            file_coverages,
        }
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    pub fn file_coverages(&self) -> &[FileCoverage] {
        &self.file_coverages
    }
}

#[doc(hidden)]
pub trait TotalCoverage {
    fn line_executed(&self) -> usize;
    fn line_total(&self) -> usize;
    fn branch_executed(&self) -> usize;
    fn branch_total(&self) -> usize;
}

#[doc(hidden)]
impl TotalCoverage for FileCoverage {
    #[cfg_attr(not(feature = "noinline"), inline)]
    fn line_executed(&self) -> usize {
        self.line_coverages
            .iter()
            .filter(|&v| v.count.map_or(false, |c| c > 0))
            .count()
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn line_total(&self) -> usize {
        self.line_coverages
            .iter()
            .filter(|&v| v.count.is_some())
            .count()
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn branch_executed(&self) -> usize {
        self.branch_coverages
            .iter()
            .filter(|&v| v.taken.unwrap_or(false))
            .count()
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn branch_total(&self) -> usize {
        self.branch_coverages
            .iter()
            .filter(|&v| v.taken.is_some())
            .count()
    }
}

#[doc(hidden)]
impl TotalCoverage for PackageCoverage {
    #[cfg_attr(not(feature = "noinline"), inline)]
    fn line_executed(&self) -> usize {
        self.file_coverages
            .iter()
            .fold(0, |sum, a| sum + a.line_executed())
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn line_total(&self) -> usize {
        self.file_coverages
            .iter()
            .fold(0, |sum, a| sum + a.line_total())
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn branch_executed(&self) -> usize {
        self.file_coverages
            .iter()
            .fold(0, |sum, a| sum + a.branch_executed())
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn branch_total(&self) -> usize {
        self.file_coverages
            .iter()
            .fold(0, |sum, a| sum + a.branch_total())
    }
}

pub trait CoverageReader {
    /// fetch the coverage information from the reader
    fn read<R: BufRead>(&self, reader: &mut R) -> Result<PackageCoverage, Error>;

    /// fetch the coverage information from file
    fn read_from_file(&self, path: &Path) -> Result<PackageCoverage, Error> {
        let f = fs::File::open(path)
            .chain_err(|| format!("Failed to open coverage file {:?}", path))?;
        let capacity = f.metadata().map(|m| m.len() as usize + 1).unwrap_or(8192);
        let mut reader = BufReader::with_capacity(capacity, f);
        self.read(&mut reader)
    }
}

pub trait CoverageWriter {
    /// save coverage information into the writer
    fn write<W: Write>(&self, data: &PackageCoverage, writer: &mut W) -> Result<(), Error>;

    /// save coverage information into the file
    fn write_to_file(&self, data: &PackageCoverage, path: &Path) -> Result<(), Error> {
        let f = fs::File::create(path).chain_err(|| format!("Failed to open file {:?}", path))?;
        let mut writer = BufWriter::new(f);
        self.write(&data, &mut writer)
    }
}
