//! Tiny shared types for SemanticAnt. Standard library only, no model, no language hard-coded.
//!
//! * [`Span`]: a byte range into the original text.
//! * [`LangTag`]: a syntactically validated BCP 47 tag; never defaulted.
//! * [`Reason`] and [`Verdict`]: every result carries the reasons for it.
//! * [`Provenance`]: where data came from, under what licence, with credit.
//! * [`pack`]: the data-pack manifest types and their validation (parsing and
//!   checksumming live in a later crate, so this one stays dependency-free).
//!
//! Stable identifiers are deliberately absent: global identity belongs to OntologicAnt.
#![forbid(unsafe_code)]

mod error;
mod lang;
pub mod pack;
mod provenance;
mod reason;
mod span;

pub use error::CoreError;
pub use lang::LangTag;
pub use provenance::Provenance;
pub use reason::{Reason, Verdict};
pub use span::Span;
