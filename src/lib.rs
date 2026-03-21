#![forbid(unsafe_code)]

pub mod builder;
pub mod error;
pub mod filter;
pub mod hashing;
pub mod params;

pub use builder::RibbonBuilder;
pub use error::{BuildError, ConstructionFailure, ParamError};
pub use filter::RibbonFilter;
pub use params::{Mode, Params};

#[cfg(test)]
mod tests;
