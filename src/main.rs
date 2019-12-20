use argparse::{ArgumentParser, Print, Store, StoreOption};
use error_chain::ChainedError;
use std::env;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process;

use rust_covfix::error::*;
use rust_covfix::{lcov::LcovParser, CoverageReader, CoverageWriter, Fixer};

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e.display_chain());
        process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let options = Arguments::parse();
    let root_dir = options.root.unwrap_or_else(find_root_dir);

    let parser = LcovParser::new(root_dir);
    let fixer = Fixer::new().chain_err(|| "Failed to initialize fixer")?;

    let mut coverage = parser.read_from_file(options.input_file)?;
    fixer.fix(&mut coverage)?;

    if let Some(file) = options.output_file {
        parser.write_to_file(&coverage, file)?;
    } else {
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
}

impl Arguments {
    fn parse() -> Arguments {
        let mut args = Arguments {
            root: None,
            input_file: PathBuf::new(),
            output_file: None,
        };

        let mut ap = ArgumentParser::new();
        ap.set_description("Rust coverage fixer");
        ap.refer(&mut args.input_file)
            .required()
            .add_argument("file", Store, "coverage file");
        ap.add_option(
            &["-v", "--version"],
            Print(env!("CARGO_PKG_VERSION").to_owned()),
            "display version",
        );
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

        ap.parse_args_or_exit();
        drop(ap);

        args.validate();
        args
    }

    fn validate(&mut self) {
        if let Some(ref root) = self.root {
            if !root.is_dir() {
                panic!("Directory not found: \"{}\"", root.display());
            }
        }

        if !self.input_file.is_file() {
            panic!("Input file not found: \"{}\"", self.input_file.display());
        }
    }
}

fn find_root_dir() -> PathBuf {
    let mut path = env::current_dir().expect("cannot detect the current directory.");
    path.push("target");

    if path.is_dir() {
        path.pop();
        return path;
    }

    path.pop();

    while path.pop() {
        path.push("target");

        if path.is_dir() {
            path.pop();
            return path;
        }

        path.pop();
    }

    panic!("cannot find the project root directory.\nDid you run `cargo test` at first?");
}
