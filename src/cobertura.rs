use quick_xml::{
    events::{attributes::Attribute as XMLAttribute, BytesStart, Event as XMLEvent},
    Reader as XMLReader, Writer as XMLWriter,
};
use std::borrow::Cow;
use std::ffi::OsString;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

use crate::coverage::{
    CoverageReader, CoverageWriter, FileCoverage, LineCoverage, PackageCoverage, TotalCoverage,
};
use crate::error::*;

macro_rules! unwrap_or_continue {
    ($val:expr) => {
        match $val {
            Some(val) => val,
            None => continue,
        }
    };
}

macro_rules! xml_tag {
    ($name:expr, (
        $($key:expr => $value:expr,)*
    )) => {{
        #[allow(unused_mut)]
        let mut bs = BytesStart::borrowed_name($name.as_bytes());
        $(
            let attribute = XMLAttribute {
                key: $key.as_bytes(),
                value: $value.into_xml_value()
            };
            bs.push_attribute(attribute);
        )*
        XMLEvent::Start(bs)
    }};
    ($name:expr, (
        $($key:expr => $value:expr),*
    )) => {
        xml_tag!($name, ($($key => $value,)*))
    };
    ($name:expr) => {{
        xml_tag!($name, ())
    }};
}

trait IntoXMLValue<'a> {
    fn into_xml_value(self) -> Cow<'a, [u8]>;
}

impl<'a> IntoXMLValue<'a> for u64 {
    fn into_xml_value(self) -> Cow<'a, [u8]> {
        self.to_string().into_bytes().into()
    }
}

impl<'a> IntoXMLValue<'a> for u32 {
    fn into_xml_value(self) -> Cow<'a, [u8]> {
        self.to_string().into_bytes().into()
    }
}

impl<'a> IntoXMLValue<'a> for String {
    fn into_xml_value(self) -> Cow<'a, [u8]> {
        self.into_bytes().into()
    }
}

impl<'a> IntoXMLValue<'a> for &'a str {
    fn into_xml_value(self) -> Cow<'a, [u8]> {
        Cow::Borrowed(self.as_bytes())
    }
}

impl<'a> IntoXMLValue<'a> for &'a [u8] {
    fn into_xml_value(self) -> Cow<'a, [u8]> {
        Cow::Borrowed(self)
    }
}

impl<'a> IntoXMLValue<'a> for &'a Path {
    fn into_xml_value(self) -> Cow<'a, [u8]> {
        match self.to_string_lossy() {
            Cow::Owned(v) => v.into_bytes().into(),
            Cow::Borrowed(v) => Cow::Borrowed(v.as_bytes()),
        }
    }
}

struct State {
    inside_source_tag: bool,
    source_dirs: Vec<PathBuf>,
}

pub struct CoberturaParser {
    root: PathBuf,
}

impl CoverageReader for CoberturaParser {
    fn read<R: BufRead>(&self, reader: &mut R) -> Result<PackageCoverage, Error> {
        let mut reader = XMLReader::from_reader(reader);
        let mut buf = Vec::new();
        let mut file_coverages = Vec::new();
        let mut state = State {
            inside_source_tag: false,
            source_dirs: vec![],
        };

        loop {
            match reader.read_event(&mut buf) {
                Ok(XMLEvent::Text(t)) if state.inside_source_tag => {
                    let bytes = t.escaped();
                    if path_is_valid(bytes) {
                        let path = bytes_to_path(bytes);
                        state.source_dirs.push(path)
                    }
                }
                Ok(XMLEvent::Start(ref t)) => match t.local_name() {
                    b"source" => {
                        state.inside_source_tag = true;
                    }
                    b"class" => {
                        let filename = unwrap_or_continue!(extract_filename_from_tag(t));
                        let path = self
                            .find_file_in_dirs(&state.source_dirs, &filename)
                            .unwrap_or(filename);

                        let line_covs = fetch_line_coverages(&mut reader);
                        let file_cov = FileCoverage::new(path, line_covs, vec![]);
                        file_coverages.push(file_cov);
                    }
                    _ => {}
                },
                Ok(XMLEvent::End(ref t)) => match t.local_name() {
                    b"sources" => {
                        state.inside_source_tag = false;
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

        let result = PackageCoverage::new(file_coverages);

        Ok(result)
    }
}

impl CoverageWriter for CoberturaParser {
    fn write<W: Write>(&self, data: &PackageCoverage, writer: &mut W) -> Result<(), Error> {
        let mut output_buffer: Vec<u8> = Vec::with_capacity(4096);
        let mut xml_writer = XMLWriter::new(&mut output_buffer);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let line_rate = data.line_executed() as f64 / data.line_total() as f64;
        let line_rate = format!("{:.3}", line_rate);

        xml_writer.write(b"<?xml version=\"1.0\">")?;
        xml_writer.write(
            b"<!DOCTYPE coverage SYSTEM 'http://cobertura.sourceforge.net/xml/coverage-03.dtd'>\n",
        )?;
        xml_writer.write_event(xml_tag!("coverage", (
            "line-rate" => line_rate.as_bytes(),
            "version" => "1.9",
            "timestamp" => timestamp
        )))?;

        self.write_sources(&mut xml_writer)?;

        xml_writer.write_event(xml_tag!("packages"))?;
        xml_writer.write_event(xml_tag!("package", (
            "name" => data.name(),
            "line-rate" => line_rate,
            "branch-rate" => "1.0",
            "complexity" => "1.0"
        )))?;
        xml_writer.write_event(xml_tag!("classes"))?;

        for file_cov in data.file_coverages() {
            self.write_file_coverage(&mut xml_writer, file_cov)?;
        }

        xml_writer.write(b"</classes>")?;
        xml_writer.write(b"</package>")?;
        xml_writer.write(b"</packages>")?;
        xml_writer.write(b"</coverage>")?;

        writer.write_all(xml_writer.into_inner())?;

        Ok(())
    }
}

impl CoberturaParser {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    fn find_file_in_dirs(&self, dirs: &[PathBuf], filename: &Path) -> Option<PathBuf> {
        if filename.is_file() {
            return Some(filename.to_owned());
        }

        for ref dir in dirs {
            let path = if dir.is_absolute() {
                dir.join(filename)
            } else {
                let mut path = self.root.join(dir);
                path.push(filename);
                path
            };
            if path.is_file() {
                return Some(path);
            }
        }

        None
    }

    fn write_sources(&self, writer: &mut XMLWriter<&mut Vec<u8>>) -> Result<(), Error> {
        writer.write_event(xml_tag!("sources"))?;
        writer.write_event(xml_tag!("source"))?;

        writer.write(self.root.as_os_str().to_str().unwrap().as_bytes())?;

        writer.write(b"</source>")?;
        writer.write(b"</sources>")?;

        Ok(())
    }

    fn write_file_coverage(
        &self,
        writer: &mut XMLWriter<&mut Vec<u8>>,
        file_cov: &FileCoverage,
    ) -> Result<(), Error> {
        let filename = file_cov
            .path()
            .file_name()
            .unwrap_or("unknown".as_ref())
            .to_string_lossy();
        let name = if cfg!(windows) {
            filename.as_ref().replace("\\", "_")
        } else {
            filename.as_ref().replace("/", "_")
        };

        let path = file_cov
            .path()
            .strip_prefix(&self.root)
            .unwrap_or_else(|_| file_cov.path());

        let line_rate = file_cov.line_executed() as f64 / file_cov.line_total() as f64;
        let line_rate = format!("{:.3}", line_rate);

        writer.write_event(xml_tag!("class", (
            "name" => name,
            "filename" => path,
            "line-rate" => line_rate
        )))?;

        writer.write_event(xml_tag!("lines"))?;

        for line_cov in file_cov.line_coverages() {
            self.write_line_coverage(writer, line_cov)?;
        }

        writer.write(b"</lines>")?;
        writer.write(b"</class>")?;

        Ok(())
    }

    fn write_line_coverage(
        &self,
        writer: &mut XMLWriter<&mut Vec<u8>>,
        line_cov: &LineCoverage,
    ) -> Result<(), Error> {
        let tag = format!(
            r#"<line number="{}" hits="{}">"#,
            line_cov.line_number + 1,
            line_cov.count
        );
        writer.write(tag.as_bytes())?;

        Ok(())
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

fn fetch_line_coverages<R: BufRead>(reader: &mut XMLReader<R>) -> Vec<LineCoverage> {
    let mut buf = Vec::new();
    let mut line_covs = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(XMLEvent::Empty(ref t)) => {
                if t.local_name() == b"line" {
                    if let Some(line_cov) = extract_line_coverage_from_tag(t) {
                        line_covs.push(line_cov);
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
            _ => {}
        }
    }

    line_covs
}

fn extract_line_coverage_from_tag(tag: &BytesStart) -> Option<LineCoverage> {
    let mut line = std::usize::MAX;
    let mut count = std::u32::MAX;

    for attribute in tag.attributes() {
        match attribute {
            Ok(XMLAttribute {
                key: b"number",
                value,
            }) => {
                let value = std::str::from_utf8(value.as_ref())
                    .ok()
                    .and_then(|s| s.parse::<usize>().ok().map(|v| v.saturating_sub(1)));

                line = match value {
                    Some(v) => v,
                    None => return None,
                };
            }
            Ok(XMLAttribute {
                key: b"hits",
                value,
            }) => {
                let value = std::str::from_utf8(value.as_ref())
                    .ok()
                    .and_then(|s| s.parse::<u32>().ok().map(|v| v.saturating_sub(1)));

                count = match value {
                    Some(v) => v,
                    None => return None,
                };
            }
            _ => {}
        }
    }

    if line != std::usize::MAX && count != std::u32::MAX {
        Some(LineCoverage {
            line_number: line,
            count,
        })
    } else {
        None
    }
}

#[cfg(unix)]
fn bytes_to_path(bytes: &[u8]) -> PathBuf {
    let s = OsString::from_vec(Vec::from(bytes));
    PathBuf::from(s)
}

#[cfg(not(unix))]
fn bytes_to_path(bytes: &[u8]) -> PathBuf {
    let s = std::str::from_utf8(bytes).unwrap().to_owned();
    PathBuf::from(s)
}

fn path_is_valid(bytes: &[u8]) -> bool {
    for b in bytes {
        if *b >= 0x20u8 {
            return true;
        }
    }

    false
}
