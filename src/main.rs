#[macro_use]
extern crate rust_covfix;

use argparse::{ArgumentParser, Print, Store, StoreOption, StoreTrue};
use error_chain::{bail, ChainedError};
use std::env;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::{self, Command};

use rust_covfix::error::*;
use rust_covfix::rule;
use rust_covfix::{parser::LcovParser, CoverageFixer, CoverageReader, CoverageWriter};

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e.display_chain());
        process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let options = Arguments::parse()?;

    if options.verbose {
        rust_covfix::set_verbosity(4);
    } else {
        rust_covfix::set_verbosity(3);
    }

    let root_dir = options
        .root
        .clone()
        .or_else(find_root_dir)
        .ok_or("cannot find the project root directory. Did you run `cargo test` at first?")?;

    debugln!("Project root directory: {:?}", root_dir);

    let parser = LcovParser::new(root_dir);

    let fixer = match options.rules {
        Some(ref rule_str) => {
            let mut rules = vec![];
            for segment in rule_str.split(',').filter(|v| !v.is_empty()) {
                rules.push(rule::from_str(segment)?);
            }
            CoverageFixer::with_rules(rules)
        }
        None => CoverageFixer::default(),
    };

    debugln!("Reading data file {:?}", options.input_file);

    let mut coverage = parser
        .read_from_file(&options.input_file)
        .chain_err(|| format!("Failed to read coverage from {:?}", options.input_file))?;

    debugln!("Found {} entries", coverage.file_coverages().len());

    if !options.nofix {
        fixer
            .fix(&mut coverage)
            .chain_err(|| "Failed to fix coverage")?;
    }

    if let Some(file) = options.output_file {
        debugln!("Writing coverage to {:?}", file);
        parser
            .write_to_file(&coverage, &file)
            .chain_err(|| format!("Failed to save coverage into file {:?}", file))?;
    } else {
        debugln!("Writing coverage to stdout");
        let stdout = std::io::stdout();
        let mut writer = BufWriter::new(stdout.lock());
        parser.write(&coverage, &mut writer)?;
    }

    Ok(())
}

struct Arguments {
    input_file: PathBuf,
    output_file: Option<PathBuf>,
    root: Option<PathBuf>,
    rules: Option<String>,
    nofix: bool,
    verbose: bool,
}

impl Arguments {
    fn parse() -> Result<Arguments, Error> {
        let mut args = Arguments {
            root: None,
            input_file: PathBuf::new(),
            output_file: None,
            rules: None,
            nofix: false,
            verbose: false,
        };

        let mut ap = ArgumentParser::new();
        ap.set_description("Rust coverage fixer");
        ap.refer(&mut args.input_file)
            .required()
            .add_argument("file", Store, "coverage file");
        ap.add_option(
            &["-V", "--version"],
            Print(env!("CARGO_PKG_VERSION").to_owned()),
            "display version",
        );
        ap.refer(&mut args.verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "verbose output");
        ap.refer(&mut args.nofix)
            .add_option(&["-n", "--no-fix"], StoreTrue, "do not fix coverage");
        ap.refer(&mut args.output_file).metavar("FILE").add_option(
            &["-o", "--output"],
            StoreOption,
            "output file name (default: stdout)",
        );
        ap.refer(&mut args.root).metavar("DIR").add_option(
            &["--root"],
            StoreOption,
            "project root directory",
        );
        ap.refer(&mut args.rules).metavar("STR[,STR..]").add_option(
            &["--rules"],
            StoreOption,
            "use specified rules to fix coverages. Valid names are [close, test, loop, derive]",
        );

        ap.parse_args_or_exit();
        drop(ap);

        args.validate().chain_err(|| "Argument validation failed")?;
        Ok(args)
    }

    fn validate(&mut self) -> Result<(), Error> {
        if let Some(ref root) = self.root {
            if !root.is_dir() {
                bail!("Directory not found: {:?}", root);
            }
        }

        if !self.input_file.is_file() {
            bail!("Input file not found: {:?}", self.input_file);
        }

        Ok(())
    }
}

fn find_root_dir() -> Option<PathBuf> {
    if let Some(mut target_dir) = find_cargo_target_dir() {
        target_dir.pop();
        return Some(target_dir);
    }

    let mut path = env::current_dir().expect("cannot detect the current directory.");
    path.push("target");

    if path.is_dir() {
        path.pop();
        return Some(path);
    }

    path.pop();

    while path.pop() {
        path.push("target");

        if path.is_dir() {
            path.pop();
            return Some(path);
        }

        path.pop();
    }

    None
}

fn find_cargo_target_dir() -> Option<PathBuf> {
    let output = Command::new("cargo")
        .args(&["metadata", "--format-version", "1"])
        .output()
        .ok()?;

    let stdout = unsafe { String::from_utf8_unchecked(output.stdout) };
    let start = stdout.rfind("\"target_directory\":")? + 20;
    let end = start + stdout[start..].find("\"")?;
    Some(PathBuf::from(&stdout[start..end]))
}
