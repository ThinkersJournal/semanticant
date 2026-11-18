use std::fmt;

/// Why a value failed validation. Every variant names the offending input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// A span with `start > end`.
    InvalidSpan {
        /// Start offset given.
        start: usize,
        /// End offset given.
        end: usize,
    },
    /// A string that is not a well-formed BCP 47 language tag.
    InvalidLangTag(String),
    /// A provenance or manifest field that must not be empty.
    EmptyField(&'static str),
    /// A licence that is not a plausible SPDX expression.
    InvalidLicence(String),
    /// A date that is not `YYYY-MM-DD`.
    InvalidDate(String),
    /// A checksum that is not 64 lowercase hex digits.
    InvalidChecksum(String),
    /// A pack file name that is empty, absolute, or escapes the pack directory.
    InvalidFileName(String),
    /// Two manifest entries with the same file name.
    DuplicateFile(String),
    /// A manifest whose schema version this crate does not know.
    UnsupportedSchema(u32),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpan { start, end } => {
                write!(f, "invalid span: start {start} > end {end}")
            }
            Self::InvalidLangTag(s) => write!(f, "invalid language tag: {s:?}"),
            Self::EmptyField(n) => write!(f, "required field is empty: {n}"),
            Self::InvalidLicence(s) => write!(f, "invalid licence expression: {s:?}"),
            Self::InvalidDate(s) => write!(f, "invalid date (want YYYY-MM-DD): {s:?}"),
            Self::InvalidChecksum(s) => {
                write!(f, "invalid checksum (want 64 lowercase hex): {s:?}")
            }
            Self::InvalidFileName(s) => write!(f, "invalid pack file name: {s:?}"),
            Self::DuplicateFile(s) => write!(f, "duplicate pack file: {s:?}"),
            Self::UnsupportedSchema(v) => write!(f, "unsupported pack schema version: {v}"),
        }
    }
}

impl std::error::Error for CoreError {}
