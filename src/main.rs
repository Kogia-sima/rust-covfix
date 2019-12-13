use std::env;
use std::path::PathBuf;
use std::process;

use rust_covfix::CoberturaParser;
use rust_covfix::{fix_coverage, CoverageReader, CoverageWriter};

fn main() {
    let options = process_args();

    let root_dir = options.root.unwrap_or_else(|| find_root_dir());

    let cf = CoberturaParser::new(root_dir);
    let mut coverage = cf.load_coverages(&options.target_file);
    fix_coverage(&mut coverage);
    cf.save_coverages(&options.target_file, &coverage);
}

struct Arguments {
    root: Option<PathBuf>,
    target_file: PathBuf,
}

fn process_args() -> Arguments {
    let mut root = None;
    let mut target_file = PathBuf::new();
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
                        root = Some(PathBuf::from(arg));
                        continue;
                    }
                }

                eprintln!("error: --root option requires an argument.\n");
                show_usage();
            }
            positional => {
                if target_file.as_os_str().is_empty() {
                    eprintln!("error: You cannot specify multiple targets.\n");
                    show_usage();
                }
                target_file = PathBuf::from(positional);
            }
        }
    }

    if target_file.as_os_str().is_empty() {
        eprintln!("error: specify target file.\n");
        show_usage();
    }
    if !target_file.exists() {
        eprintln!(
            "error: target file {} does not exist.\n",
            target_file.display()
        );
        show_usage();
    }

    Arguments { root, target_file }
}

fn show_usage() {
    println!(
        concat!(
            "{} {}\n",
            "Copyright (c): 2019 {}\n\n",
            "Usage:\n",
            "  {} [OPTIONS] <file>\n\n",
            "Options:\n",
            "  -h --help        show usage\n",
            "  -v --version     output version information\n",
            "     --root <dir>  specify project root directory"
        ),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS")
    );

    process::exit(1);
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
