use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use crate::common::{
    BranchCoverage, CoverageReader, CoverageWriter, FileCoverage, LineCoverage, PackageCoverage,
    SupportedFile, TotalCoverage,
};

/// Enumeration representing each line in 'lcov.info'
pub enum RawData<'a> {
    /// Test Name
    TN(&'a str),

    /// Source File
    SF(&'a Path),

    /// Function
    FN(usize, &'a str),

    /// Fn Data
    FNDA(u32, &'a str),

    /// \# FN Found
    FNF(u32),

    /// \# Fn Executed
    FNH(u32),

    /// Executions for some Line
    DA(usize, u32),

    /// \# Lines Found
    LF(u32),

    /// \# Lines Executed
    LH(u32),

    /// Branch coverage information
    BRDA(usize, usize, usize, bool),

    /// \# Branches Found
    BRF(u32),

    /// \# Branches Executed
    BRH(u32),

    /// End of Record
    EndOfRecord,
}

pub struct LcovParser {
    root: PathBuf,
}

impl SupportedFile for LcovParser {
    fn is_supported(&self, path: &Path) -> bool {
        match path.file_name() {
            Some(filename) => filename == "lcov.info",
            None => false,
        }
    }
}

impl CoverageReader for LcovParser {
    fn read<R: BufRead>(&self, reader: &mut R) -> PackageCoverage {
        let mut line_buf = String::with_capacity(120);
        let mut line_coverages = Vec::new();
        let mut branch_coverages = Vec::new();
        let mut file_coverages = Vec::new();
        let mut filename = PathBuf::new();
        let mut testname = String::new();

        while let Ok(_) = reader.read_line(&mut line_buf) {
            let raw_data = match self.parse_line(&line_buf) {
                Some(raw_data) => raw_data,
                None => continue,
            };

            match raw_data {
                RawData::TN(name) => testname = name.into(),
                RawData::SF(file) => {
                    if !self.check_path(file) {
                        panic!("source file not found.: \"{}\"", file.display());
                    }
                    filename = file.into();
                }
                RawData::DA(line, count) => {
                    line_coverages.push(LineCoverage {
                        line_number: line,
                        count,
                    });
                }
                RawData::BRDA(line, block, branch, taken) => {
                    branch_coverages.push(BranchCoverage {
                        line_number: line,
                        block_number: Some(block),
                        branch_number: Some(branch),
                        taken,
                    });
                }
                RawData::EndOfRecord => {
                    let file_coverage = FileCoverage::new(
                        filename.clone(),
                        line_coverages.drain(..).collect(),
                        branch_coverages.drain(..).collect(),
                    );
                    file_coverages.push(file_coverage);

                    debug_assert!(line_coverages.is_empty());
                    debug_assert!(branch_coverages.is_empty());
                }
                _ => {}
            }
        }

        PackageCoverage::new(testname, file_coverages)
    }
}

impl CoverageWriter for LcovParser {
    fn write<W: Write>(&self, data: &PackageCoverage, writer: &mut W) {
        self.write_package_coverage(writer, data);
    }
}

impl LcovParser {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    fn parse_line<'a>(&self, line: &'a str) -> Option<RawData<'a>> {
        if line == "end_of_record" {
            return Some(RawData::EndOfRecord);
        }

        let end = match line.find(':') {
            Some(end) => end,
            None => return None,
        };

        let prefix = &line[0..end];
        let mut contents = line[end + 1..].split(',');

        return match prefix {
            "TN" => Some(RawData::TN(contents.next().unwrap_or(""))),
            "SF" => Some(RawData::SF(Path::new(contents.next()?))),
            "FN" => {
                let line = contents.next()?.parse().ok()?;
                let name = contents.next()?;
                Some(RawData::FN(line, name))
            }
            "FNDA" => {
                let count = contents.next()?.parse().ok()?;
                let name = contents.next()?;
                Some(RawData::FNDA(count, name))
            }
            "FNF" => Some(RawData::FNF(contents.next()?.parse().ok()?)),
            "FNH" => Some(RawData::FNH(contents.next()?.parse().ok()?)),
            "DA" => {
                let line = contents.next()?.parse().ok()?;
                let count = contents.next()?.parse().ok()?;
                Some(RawData::DA(line, count))
            }
            "LF" => Some(RawData::LF(contents.next()?.parse().ok()?)),
            "LH" => Some(RawData::LH(contents.next()?.parse().ok()?)),
            "BRDA" => {
                let line = contents.next()?.parse().ok()?;
                let block = contents.next()?.parse().ok()?;
                let branch = contents.next()?.parse().ok()?;
                let taken = contents.next()? != "-";
                Some(RawData::BRDA(line, block, branch, taken))
            }
            "BRF" => Some(RawData::BRF(contents.next()?.parse().ok()?)),
            "BRH" => Some(RawData::BRH(contents.next()?.parse().ok()?)),
            _ => None,
        };
    }

    fn check_path(&self, path: &Path) -> bool {
        let filepath = self.root.join(path);
        filepath.is_file()
    }

    fn write_package_coverage<W: Write>(&self, writer: &mut W, data: &PackageCoverage) {
        writeln!(writer, "TN:{}", data.name()).unwrap();

        for cov in data.file_coverages() {
            self.write_file_coverage(writer, cov);
        }
    }

    fn write_file_coverage<W: Write>(&self, writer: &mut W, data: &FileCoverage) {
        writeln!(writer, "SF:{}", data.path().display()).unwrap();

        for cov in data.branch_coverages() {
            self.write_branch_coverage(writer, cov);
        }

        writeln!(writer, "BRF:{}", data.branch_total()).unwrap();
        writeln!(writer, "BRH:{}", data.branch_executed()).unwrap();

        for cov in data.line_coverages() {
            self.write_line_coverage(writer, cov);
        }

        writeln!(writer, "LF:{}", data.line_total()).unwrap();
        writeln!(writer, "LH:{}", data.line_executed()).unwrap();

        writeln!(writer, "end_of_record").unwrap();
    }

    fn write_branch_coverage<W: Write>(&self, writer: &mut W, data: &BranchCoverage) {
        writeln!(
            writer,
            "BRDA:{},{},{},{}\n",
            data.line_number,
            data.block_number.unwrap_or(1),
            data.branch_number.unwrap_or(1),
            if data.taken { "1" } else { "-" }
        )
        .unwrap();
    }

    fn write_line_coverage<W: Write>(&self, writer: &mut W, data: &LineCoverage) {
        writeln!(writer, "DA:{},{}", data.line_number, data.count).unwrap();
    }
}
