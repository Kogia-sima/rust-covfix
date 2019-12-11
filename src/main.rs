use std::env;
use std::path::PathBuf;
use std::process;

use kcov_coverage_fix::CoberturaParser;
use kcov_coverage_fix::{fix_coverage, CoverageReader, CoverageWriter};

#[cfg(not(windows))]
const COV_DIR: &str = "target/cov";

#[cfg(windows)]
const COV_DIR: &str = "target\\cov";

fn main() {
    let options = process_args();

    let cov_dir = if let Some(mut root) = options.root {
        root.push(COV_DIR);
        if !root.exists() {
            panic!("Run `cargo kcov` before execution.");
        }
        root
    } else {
        find_cov_dir()
    };
    let file = search_coverage_file(cov_dir);

    let cf = CoberturaParser::new();
    let mut coverage = cf.load_coverages(&file);
    fix_coverage(&mut coverage);
    cf.save_coverages(&file, &coverage);
}

struct Arguments {
    root: Option<PathBuf>,
}

fn process_args() -> Arguments {
    let mut results = Arguments { root: None };
    let mut args = env::args();
    args.next().unwrap();

    while let Some(arg) = args.next() {
        match &*arg {
            "-h" | "--help" => show_usage(),
            "-v" | "--version" => {
                println!("v{}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            "--root" => {
                if let Some(arg) = args.next() {
                    if !arg.starts_with("-") {
                        results.root = Some(PathBuf::from(arg));
                        continue;
                    }
                }

                eprintln!("--root option requires an argument.");
                show_usage();
            }
            invalid_arg => {
                eprintln!("invalid argument: \"{}\"\n", invalid_arg);
                show_usage();
            }
        }
    }

    results
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
            "  -v --version: output version information\n",
            "     --root:    specify project root directory"
        ),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    );

    process::exit(1);
}

fn search_coverage_file(mut p: PathBuf) -> PathBuf {
    p.push("kcov-merged");
    p.push("cobertura.xml");

    p
}

fn find_cov_dir() -> PathBuf {
    let mut path = env::current_dir().expect("cannot detect the current directory.");
    path.push(COV_DIR);

    if path.is_dir() {
        return path;
    }

    path.pop();
    path.pop();

    while path.pop() {
        path.push(COV_DIR);

        if path.is_dir() {
            return path;
        }

        path.pop();
        path.pop();
    }

    panic!("Run `cargo kcov` before execution.");
}
