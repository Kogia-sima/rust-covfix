#![cfg_attr(docsrs, feature(doc_cfg))]

mod coverage;
pub use coverage::*;

mod fix;
pub use fix::*;

pub mod error;

#[cfg(feature = "lcov")]
#[cfg_attr(docsrs, doc(cfg(feature = "lcov")))]
mod lcov;

#[cfg(feature = "cobertura")]
#[cfg_attr(docsrs, doc(cfg(feature = "cobertura")))]
mod cobertura;

pub mod parser {
    #[cfg(feature = "lcov")]
    #[cfg_attr(docsrs, doc(cfg(feature = "lcov")))]
    pub use super::lcov::*;

    #[cfg(feature = "cobertura")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cobertura")))]
    pub use super::cobertura::*;
}
