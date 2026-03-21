#![forbid(unsafe_code)]

//! Ribbon filter crate (incremental sprint implementation).
//!
//! Guarantees in current mode (`Mode::Standard`, `w <= 64`):
//! - no false negatives for inserted keys after successful build,
//! - probabilistic false positives controlled by `r` fingerprint bits,
//! - deterministic behavior for fixed params, key-set, and hasher.
//!
//! # Example
//! ```
//! use std::collections::hash_map::DefaultHasher;
//! use std::hash::BuildHasherDefault;
//! use ribbon_filter::{Mode, Params, RibbonBuilder};
//!
//! type H = BuildHasherDefault<DefaultHasher>;
//! let params = Params::new(3000, 16, 10, Mode::Standard)
//!     .expect("valid params")
//!     .with_seed(42);
//! let builder = RibbonBuilder::new(params, H::default()).expect("builder");
//! let keys: Vec<u64> = (0..1000).collect();
//! let filter = builder.build(&keys).expect("build");
//! assert!(filter.contains(&123u64));
//! ```

pub mod builder;
pub mod error;
pub mod filter;
pub mod hashing;
pub mod params;

pub use builder::{RibbonBuilder, Scratch};
pub use error::{BuildError, ConstructionFailure, ParamError};
pub use filter::RibbonFilter;
pub use params::{Mode, Params};

#[cfg(test)]
mod tests;
