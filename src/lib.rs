#![cfg_attr(docsrs, feature(doc_cfg))]

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
