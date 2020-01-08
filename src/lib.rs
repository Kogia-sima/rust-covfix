#![cfg_attr(docsrs, feature(doc_cfg))]

//! Fix rust coverage based on source code.
//!
//! Correct coverage is useful if you want to know what kind of tests you should write
//! to avoid bugs in your project.
//! This crate offers utilities to read coverage information from file,
//! fix that information by choosing the rules, and save coverage into file.
//!
//! This crate is very customizable.
//! You can also create custom reader/writer by implementing
//! `CoverageReader`/`CoverageWriter` trait. If you want to define a new rule to
//! fix the coverage, implement `Rule` trait.

mod coverage;
pub use coverage::*;

mod fix;
pub use fix::*;

pub mod rule;

pub mod error;

#[cfg(feature = "lcov")]
#[cfg_attr(docsrs, doc(cfg(feature = "lcov")))]
mod lcov;

pub mod parser {
    #[cfg(feature = "lcov")]
    #[cfg_attr(docsrs, doc(cfg(feature = "lcov")))]
    pub use super::lcov::*;
}
