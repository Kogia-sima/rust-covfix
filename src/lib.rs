mod coverage;
pub use coverage::*;

mod fix;
pub use fix::*;

pub mod error;

#[cfg(feature = "lcov")]
mod lcov;

pub mod parser {
    #[cfg(feature = "lcov")]
    pub use super::lcov::*;
}
