use regex::Regex;
use std::fs;

use crate::common::{LineCoverage, PackageCoverage, SourceCode};

pub fn fix_coverage(data: &mut PackageCoverage) {
    let non_executable_lines = vec![
        Regex::new(r"^(?:\s*\}(?:\s*\))*(?:\s*;)?|\s*(?:\}\s*)?else(?:\s*\{)?)?\s*(?://.*)?$")
            .unwrap(),
        Regex::new(r"^\s*pub\s*struct\s*.*?\{\s*(?://.*)?$").unwrap(),
    ];

    for (file, covs) in data.iter_mut() {
        let content = fs::read_to_string(file).unwrap();
        let source = SourceCode::new(content);

        debug_assert!(
            source.total_lines() == covs.len(),
            "Internal error: line length did not match"
        );
        for (line, cov) in source.lines().zip(covs) {
            if *cov == LineCoverage::NotCovered {
                if non_executable_lines.iter().any(|r| r.is_match(line)) {
                    *cov = LineCoverage::NotExecutable;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fix_coverage;
    use crate::common::{LineCoverage, PackageCoverage};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_file_with_contents(content: &[u8]) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("Failed to create a temporary file.");
        f.write_all(content).unwrap();
        f
    }

    #[test]
    fn closing_branckets() {
        let f = create_file_with_contents(
            b"\
            if a > 0 {\n\
                b = a;\n\
            } else {\n\
                b = -a;\n\
            }\n\
        ",
        );
        let p = f.path();

        let fc = vec![LineCoverage::NotCovered; 5];

        let mut pc = PackageCoverage::new();
        pc.insert(p.to_owned(), fc);

        fix_coverage(&mut pc);

        assert_eq!(
            pc.get(p),
            Some(&vec![
                LineCoverage::NotCovered,
                LineCoverage::NotCovered,
                LineCoverage::NotExecutable,
                LineCoverage::NotCovered,
                LineCoverage::NotExecutable,
            ])
        );
    }

    #[test]
    fn struct_declaration() {
        let f = create_file_with_contents(
            b"\
            pub struct Parser<R: BufRead> {\n\
                reader: R
            }\n\
        ",
        );
        let p = f.path();

        let fc = vec![LineCoverage::NotCovered; 3];

        let mut pc = PackageCoverage::new();
        pc.insert(p.to_owned(), fc);

        fix_coverage(&mut pc);

        assert_eq!(
            pc.get(p),
            Some(&vec![
                LineCoverage::NotExecutable,
                LineCoverage::NotCovered,
                LineCoverage::NotExecutable,
            ])
        );
    }
}
