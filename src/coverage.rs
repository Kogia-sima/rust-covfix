use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::error::*;

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
    #[cfg_attr(not(feature = "noinline"), inline)]
    fn line_executed(&self) -> usize {
        self.line_coverages
            .iter()
            .filter(|&v| v.count.map_or(false, |v| v > 0))
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
        self.branch_coverages.iter().filter(|&v| v.taken).count()
    }

    #[cfg_attr(not(feature = "noinline"), inline)]
    fn branch_total(&self) -> usize {
        self.branch_coverages.len()
    }
}

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

    fn read_from_file<P: AsRef<Path>>(&self, path: P) -> Result<PackageCoverage, Error> {
        let path = path.as_ref();
        let f = fs::File::open(path)
            .chain_err(|| format!("Failed to open coverage file {:?}", path))?;
        let capacity = f.metadata().map(|m| m.len() as usize + 1).unwrap_or(8192);
        let mut reader = BufReader::with_capacity(capacity, f);
        self.read(&mut reader)
    }
}

pub trait CoverageWriter {
    fn write<W: Write>(&self, data: &PackageCoverage, writer: &mut W) -> Result<(), Error>;

    fn write_to_file<P: AsRef<Path>>(&self, data: &PackageCoverage, path: P) -> Result<(), Error> {
        let path = path.as_ref();
        let f = fs::File::create(path)
            .chain_err(|| format!("Failed to save coverage into file {:?}", path))?;
        let mut writer = BufWriter::new(f);
        self.write(&data, &mut writer)
    }
}
