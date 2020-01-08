use error_chain::error_chain;
use std::fmt;
use std::io;
use std::path::PathBuf;

error_chain! {
    // The type defined for this error
    types {
        Error, ErrorKind, ResultExt;
    }

    // Automatic conversions between this error chain and other error types
    foreign_links {
        FmtError(fmt::Error);
        IoError(io::Error);
        RegexError(regex::Error);
    }

    // Custom errors
    errors {
        SourceFileNotFound(p: PathBuf) {
            description("Source file not found"),
            display("Source file {:?} not found", p)
        }
        InvalidRuleName(name: String) {
            description("Invalid Rule name"),
            display("Invalid Rule name: {:?}", name)
        }
    }
}
