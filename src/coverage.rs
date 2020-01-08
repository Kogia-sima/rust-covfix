use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::error::*;

#[derive(Clone, Debug, PartialEq)]
pub struct LineCoverage {
    pub line_number: usize,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BranchCoverage {
    pub line_number: Option<usize>,
    pub block_number: Option<usize>,
    pub taken: bool,
}

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
        self.line_coverages.iter().filter(|&v| v.count > 0).count()
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn line_total(&self) -> usize {
        self.line_coverages.len()
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn branch_executed(&self) -> usize {
        self.branch_coverages.iter().filter(|&v| v.taken).count()
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn branch_total(&self) -> usize {
        self.branch_coverages.len()
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
    fn read<R: BufRead>(&self, reader: &mut R) -> Result<PackageCoverage, Error>;

    fn read_from_file(&self, path: &Path) -> Result<PackageCoverage, Error> {
        let f = fs::File::open(path)
            .chain_err(|| format!("Failed to open coverage file {:?}", path))?;
        let capacity = f.metadata().map(|m| m.len() as usize + 1).unwrap_or(8192);
        let mut reader = BufReader::with_capacity(capacity, f);
        self.read(&mut reader)
    }
}

pub trait CoverageWriter {
    fn write<W: Write>(&self, data: &PackageCoverage, writer: &mut W) -> Result<(), Error>;

    fn write_to_file(&self, data: &PackageCoverage, path: &Path) -> Result<(), Error> {
        let f = fs::File::create(path).chain_err(|| format!("Failed to open file {:?}", path))?;
        let mut writer = BufWriter::new(f);
        self.write(&data, &mut writer)
    }
}
