use quick_xml::{
    events::{attributes::Attribute as XMLAttribute, BytesStart, BytesEnd, Event as XMLEvent},
    Reader as XMLReader, Writer as XMLWriter,
};
use std::borrow::Cow;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

use crate::common::{CoverageReader, CoverageWriter, LineCoverage, PackageCoverage, TotalCoverage};

pub struct CoberturaParser {}

impl CoverageReader for CoberturaParser {
    fn load_coverages(&self, path: &Path) -> PackageCoverage {
        let content = fs::read_to_string(path).unwrap();
        let mut reader = XMLReader::from_str(&content);
        let mut buf = Vec::new();
        let mut inside_sources_tag = false;
        let mut source_dirs: Vec<PathBuf> = Vec::new();
        let mut results = PackageCoverage::new();

        loop {
            match reader.read_event(&mut buf) {
                Ok(XMLEvent::Text(t)) if inside_sources_tag => {
                    let bytes = t.escaped();
                    if path_is_valid(bytes) {
                        let path = bytes_to_path(bytes);
                        source_dirs.push(path)
                    }
                }
                Ok(XMLEvent::Start(ref t)) => {
                    match t.local_name() {
                        b"source" => {
                            inside_sources_tag = true;
                        }
                        b"class" => {
                            let filename = extract_filename_from_tag(t);

                            if filename.is_none() {
                                // could not find filename attribute.
                                continue;
                            }

                            let path = find_file_in_dirs(&source_dirs, &filename.unwrap());
                            if path.is_none() {
                                continue;
                            }
                            let path = path.unwrap();

                            let content = fs::read_to_string(&path).unwrap();
                            let mut coverages = vec![LineCoverage::NotExecutable; content.lines().count()];

                            fetch_line_coverages(&mut reader, &mut coverages);

                            results.insert(path, coverages);
                        }
                        _ => {}
                    }
                }
                Ok(XMLEvent::End(ref t)) => match t.local_name() {
                    b"sources" => {
                        inside_sources_tag = false;
                    }
                    b"packages" => break,
                    _ => {}
                },
                Ok(XMLEvent::Eof) => break,
                Err(e) => {
                    panic!("Error at position {}: {:?}", reader.buffer_position(), e);
                }
                _ => {}
            }
        }

        results
    }
}

impl CoverageWriter for CoberturaParser {
    fn save_coverages(&self, path: &Path, data: &PackageCoverage) {
        let content = fs::read_to_string(path).unwrap();
        let mut reader = XMLReader::from_str(&content);
        let mut buf = Vec::new();
        let mut inside_sources_tag = false;
        let mut source_dirs: Vec<PathBuf> = Vec::new();

        let mut output_buffer: Vec<u8> = Vec::with_capacity(content.len());
        let mut writer = XMLWriter::new(&mut output_buffer);

        loop {
            let event = reader.read_event(&mut buf);
            match &event {
                Ok(XMLEvent::Text(ref t)) if inside_sources_tag => {
                    let bytes = t.escaped();
                    if path_is_valid(bytes) {
                        let path = bytes_to_path(bytes);
                        source_dirs.push(path)
                    }
                }
                Ok(XMLEvent::Start(ref t)) => {
                    match t.local_name() {
                        b"source" => {
                            inside_sources_tag = true;
                        }
                        b"class" => {
                            reader.read_to_end(b"class", &mut Vec::new()).unwrap();

                            let covs = extract_filename_from_tag(t).and_then(|f| {
                                find_file_in_dirs(&source_dirs, &f)
                            }).and_then(|p| {
                                data.get(&p)
                            });

                            if covs.is_none() {
                                continue;
                            }
                            let covs = covs.unwrap();

                            let line_rate = format!("{:.3}", covs.line_rate() * 100.0);

                            let mut t2 = t.clone();
                            let mut attrs = t.attributes();
                            let attrs = attrs.with_checks(false).filter_map(|v| {
                                if let Ok(attr) = v {
                                    if attr.key == b"line-rate" {
                                        Some(XMLAttribute {
                                            key: b"line-rate",
                                            value: Cow::from(line_rate.as_bytes())
                                        })
                                    } else {
                                        Some(attr)
                                    }
                                } else {
                                    None
                                }
                            });

                            t2.clear_attributes();
                            t2.extend_attributes(attrs);

                            writer.write_event(XMLEvent::Start(t2)).unwrap();
                            writer.write(b"\n").unwrap();

                            let elem = BytesStart::borrowed(b"lines", 5);
                            let event = XMLEvent::Start(elem);
                            writer.write_event(event).unwrap();
                            writer.write(b"\n").unwrap();

                            for (i, cov) in covs.into_iter().enumerate() {
                                let hits = match cov {
                                    LineCoverage::Covered => {
                                        1
                                    },
                                    LineCoverage::NotCovered => {
                                        0
                                    },
                                    LineCoverage::NotExecutable => {
                                        continue;
                                    }
                                };

                                let content = format!("line number=\"{}\" hits=\"{}\"",
                                                      i + 1, hits);
                                let elem = BytesStart::borrowed(content.as_bytes(), 4);
                                let event = XMLEvent::Empty(elem);
                                writer.write_event(event).unwrap();
                                writer.write(b"\n").unwrap();
                            }

                            let elem = BytesEnd::borrowed(b"lines");
                            let event = XMLEvent::End(elem);
                            writer.write_event(event).unwrap();
                            writer.write(b"\n").unwrap();

                            let elem = BytesEnd::borrowed(b"class");
                            let event = XMLEvent::End(elem);
                            writer.write_event(event).unwrap();

                            continue;
                        }
                        _ => {}
                    }
                }
                Ok(XMLEvent::End(ref t)) => match t.local_name() {
                    b"sources" => {
                        inside_sources_tag = false;
                    }
                    _ => {}
                },
                Ok(XMLEvent::Eof) => break,
                Err(e) => {
                    panic!("Error at position {}: {:?}", reader.buffer_position(), e);
                }
                _ => {}
            }

            writer.write_event(event.unwrap()).unwrap();
        }

        fs::write(&path, &output_buffer)
            .unwrap_or_else(|_| panic!("Failed to save coverage into {:?}.", path));
    }
}

impl CoberturaParser {
    pub fn new() -> Self {
        Self {}
    }
}

fn fetch_line_coverages(
    reader: &mut XMLReader<&[u8]>,
    coverages: &mut Vec<LineCoverage>,
) {
    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(XMLEvent::Empty(ref t)) => {
                if t.local_name() == b"line" {
                    let (line, coverage) = extract_line_coverage_from_tag(t);
                    if let Some(line) = line {
                        coverages[line] = coverage;
                    }
                }
            }
            Ok(XMLEvent::End(ref t)) => {
                if t.local_name() == b"class" {
                    break;
                }
            }
            Ok(XMLEvent::Eof) => {
                break;
            }
            _ => {},
        }
    }
}

fn extract_filename_from_tag(tag: &BytesStart) -> Option<PathBuf> {
    for attribute in tag.attributes() {
        match attribute {
            Ok(XMLAttribute {
                key: b"filename",
                value,
            }) => {
                return Some(bytes_to_path(value.as_ref()));
            }
            _ => {}
        }
    }

    None
}

fn extract_line_coverage_from_tag(tag: &BytesStart) -> (Option<usize>, LineCoverage) {
    let mut line = None;
    let mut coverage = LineCoverage::NotExecutable;
    for attribute in tag.attributes() {
        match attribute {
            Ok(XMLAttribute {
                key: b"number",
                value,
            }) => {
                if let Ok(s) = std::str::from_utf8(value.as_ref()) {
                    line = s.parse::<usize>().ok().map(|v| v.saturating_sub(1));
                }
            }
            Ok(XMLAttribute {
                key: b"hits",
                value,
            }) => {
                coverage = match value.as_ref() {
                    b"0" => LineCoverage::NotCovered,
                    _ => LineCoverage::Covered,
                }
            }
            _ => {}
        }
    }

    (line, coverage)
}

fn find_file_in_dirs(dirs: &[PathBuf], filename: &Path) -> Option<PathBuf> {
    for ref dir in dirs {
        let path = dir.join(filename);
        if path.is_file() {
            return Some(path);
        }
    }

    None
}

#[cfg(unix)]
fn bytes_to_path(bytes: &[u8]) -> PathBuf {
    let s = OsString::from_vec(Vec::from(bytes));
    PathBuf::from(s)
}

#[cfg(not(unix))]
fn bytes_to_path(bytes: &[u8]) -> PathBuf {
    let s = std::str::from_utf8(bytes);
    PathBuf::from(s)
}

fn path_is_valid(bytes: &[u8]) -> bool {
    for b in bytes {
        if *b >= 0x20u8 {
            return true;
        }
    };

    false
}
