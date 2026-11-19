//! Data-pack loader. See the crate README for the format.
#![forbid(unsafe_code)]

mod error;
mod manifest;
mod pack;

pub use error::PackError;
pub use manifest::{manifest_to_toml, parse_manifest};
pub use pack::{Pack, sha256_hex};
