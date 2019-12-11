pub mod common;
pub mod cobertura;
pub mod fix;

use std::env;
use std::path::PathBuf;
use std::process;

use cobertura::CoberturaParser;
use common::{CoverageReader, CoverageWriter};

fn main() {
    process_args();
    let file = search_coverage_file();

    let cf = CoberturaParser::new();
    let mut coverage = cf.load_coverages(&file);
    fix::fix_coverage(&mut coverage);
    cf.save_coverages(&file, &coverage);
}

fn process_args() {
    let mut args = env::args();
    args.next().unwrap();

    for arg in args {
        match &*arg {
            "-h" | "--help" => show_usage(),
            "-v" | "--version" => {
                println!("v{}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            invalid_arg => {
                eprintln!("invalid argument: \"{}\"\n", invalid_arg);
                show_usage();
            }
        }
    }
}

fn show_usage() {
    println!(
        concat!(
            "kcov-coverage-fix {}\n",
            "Copyright (c): 2019 {}\n\n",
            "Usage:\n",
            "  kcov-coverage-fix [OPTIONS]\n\n",
            "Options:\n",
            "  -h --help:    show usage\n",
            "  -v --version: output version information",
        ),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    );

    process::exit(1);
}

fn search_coverage_file() -> PathBuf {
    let mut p = find_target_dir();
    p.push("cov");
    p.push("kcov-merged");
    p.push("cobertura.xml");

    p
}

fn find_target_dir() -> PathBuf {
    let mut path = env::current_dir().expect("cannot detect the current directory.");
    path.push("target");
    
    if path.is_dir() {
        return path;
    }

    path.pop();

    while path.pop() {
        path.push("target");

        if path.is_dir() {
            return path;
        }

        path.pop();
    }

    panic!("Run `cargo kcov` before execution.");
}
