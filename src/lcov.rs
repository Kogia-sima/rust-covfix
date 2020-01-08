use error_chain::bail;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use crate::coverage::{
    BranchCoverage, CoverageReader, CoverageWriter, FileCoverage, LineCoverage, PackageCoverage,
    TotalCoverage,
};
use crate::error::*;

/// Enumeration representing each line in 'lcov.info'
enum RawData<'a> {
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

impl CoverageReader for LcovParser {
    fn read<R: BufRead>(&self, reader: &mut R) -> Result<PackageCoverage, Error> {
        let mut line_buf = String::with_capacity(120);
        let mut line_coverages = Vec::new();
        let mut branch_coverages = Vec::new();
        let mut file_coverages = Vec::new();
        let mut filename = PathBuf::new();
        let mut testname = String::new();

        while let Ok(n) = reader.read_line(&mut line_buf) {
            if n == 0 {
                break;
            }

            let raw_data = match self.parse_line(&line_buf) {
                Some(raw_data) => raw_data,
                None => {
                    line_buf.clear();
                    continue;
                }
            };

            match raw_data {
                RawData::TN(name) => testname = name.into(),
                RawData::SF(file) => {
                    filename = file.into();
                }
                RawData::DA(line, count) => {
                    if line > 0 {
                        line_coverages.push(LineCoverage {
                            line_number: line - 1,
                            count,
                        });
                    }
                }
                RawData::BRDA(line, block, _, taken) => {
                    branch_coverages.push(BranchCoverage {
                        line_number: Some(line.saturating_sub(1)),
                        block_number: Some(block),
                        taken,
                    });
                }
                RawData::EndOfRecord => {
                    let filepath = self.root.join(&filename);
                    if !filepath.is_file() {
                        bail!(ErrorKind::SourceFileNotFound(filepath));
                    }

                    let file_coverage = FileCoverage::new(
                        filepath,
                        line_coverages.drain(..).collect(),
                        branch_coverages.drain(..).collect(),
                    );
                    file_coverages.push(file_coverage);

                    debug_assert!(line_coverages.is_empty());
                    debug_assert!(branch_coverages.is_empty());
                }
                _ => {}
            }

            line_buf.clear();
        }

        Ok(PackageCoverage::with_test_name(testname, file_coverages))
    }
}

impl CoverageWriter for LcovParser {
    #[cfg_attr(not(feature = "noinline"), inline)]
    fn write<W: Write>(&self, data: &PackageCoverage, writer: &mut W) -> Result<(), Error> {
        self.write_package_coverage(writer, data)
    }
}

impl LcovParser {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    fn parse_line<'a>(&self, line: &'a str) -> Option<RawData<'a>> {
        let line = line.trim_end();
        if line == "end_of_record" {
            return Some(RawData::EndOfRecord);
        }

        let end = match line.find(':') {
            Some(end) => end,
            None => return None,
        };

        let prefix = &line[0..end];
        let mut contents = line[end + 1..].split(',');

        match prefix {
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
                if line == 0 {
                    return None;
                }

                let block = contents.next()?.parse().ok()?;
                let branch = contents.next()?.parse().ok()?;
                let taken = contents.next()? != "-";
                Some(RawData::BRDA(line, block, branch, taken))
            }
            "BRF" => Some(RawData::BRF(contents.next()?.parse().ok()?)),
            "BRH" => Some(RawData::BRH(contents.next()?.parse().ok()?)),
            _ => None,
        }
    }

    fn write_package_coverage<W: Write>(
        &self,
        writer: &mut W,
        data: &PackageCoverage,
    ) -> Result<(), Error> {
        writeln!(writer, "TN:{}", data.name())?;

        for cov in data.file_coverages() {
            self.write_file_coverage(writer, cov)?;
        }

        Ok(())
    }

    fn write_file_coverage<W: Write>(
        &self,
        writer: &mut W,
        data: &FileCoverage,
    ) -> Result<(), Error> {
        let path = data.path().strip_prefix(&self.root).unwrap();
        writeln!(writer, "SF:{}", path.display())?;

        let mut current_line = Some(0);
        let mut count = 0;
        for cov in data.branch_coverages() {
            if cov.line_number == current_line {
                count += 1;
            } else {
                count = 0;
            }
            current_line = cov.line_number;

            self.write_branch_coverage(writer, cov, count)?;
        }

        writeln!(writer, "BRF:{}", data.branch_total())?;
        writeln!(writer, "BRH:{}", data.branch_executed())?;

        for cov in data.line_coverages() {
            self.write_line_coverage(writer, cov)?;
        }

        writeln!(writer, "LF:{}", data.line_total())?;
        writeln!(writer, "LH:{}", data.line_executed())?;

        writeln!(writer, "end_of_record")?;

        Ok(())
    }

    fn write_branch_coverage<W: Write>(
        &self,
        writer: &mut W,
        data: &BranchCoverage,
        branch_number: usize,
    ) -> Result<(), Error> {
        if let Some(line_number) = data.line_number {
            writeln!(
                writer,
                "BRDA:{},{},{},{}",
                line_number + 1,
                data.block_number.unwrap_or(0),
                branch_number,
                if data.taken { "1" } else { "-" }
            )?;
        }

        Ok(())
    }

    fn write_line_coverage<W: Write>(
        &self,
        writer: &mut W,
        data: &LineCoverage,
    ) -> Result<(), Error> {
        writeln!(writer, "DA:{},{}", data.line_number + 1, data.count)?;

        Ok(())
    }
}
