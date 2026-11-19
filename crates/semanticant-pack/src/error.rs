use semanticant_core::CoreError;
use std::{fmt, path::PathBuf};

/// Why a pack could not be loaded.
#[derive(Debug)]
pub enum PackError {
    /// A file could not be read.
    Io {
        /// The path involved.
        path: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },
    /// `pack.toml` is not valid TOML.
    Toml(String),
    /// A required manifest key is missing or has the wrong type.
    Field(&'static str),
    /// The manifest parsed but failed validation (no licence, bad file name, ...).
    Invalid(CoreError),
    /// A listed file is absent from the pack.
    Missing(String),
    /// A listed file is a symlink, which packs may not use.
    Symlink(String),
    /// A listed file's bytes do not match its recorded SHA-256.
    ChecksumMismatch {
        /// The file.
        file: String,
        /// Digest recorded in the manifest.
        expected: String,
        /// Digest of the bytes actually read.
        actual: String,
    },
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "cannot read {}: {source}", path.display()),
            Self::Toml(e) => write!(f, "pack.toml is not valid TOML: {e}"),
            Self::Field(k) => write!(f, "pack.toml: missing or mistyped key `{k}`"),
            Self::Invalid(e) => write!(f, "pack manifest refused: {e}"),
            Self::Missing(n) => write!(f, "listed file is missing: {n}"),
            Self::Symlink(n) => write!(f, "listed file is a symlink: {n}"),
            Self::ChecksumMismatch {
                file,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "checksum mismatch for {file}: manifest says {expected}, file is {actual}"
                )
            }
        }
    }
}

impl std::error::Error for PackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Invalid(e) => Some(e),
            _ => None,
        }
    }
}

impl From<CoreError> for PackError {
    fn from(e: CoreError) -> Self {
        Self::Invalid(e)
    }
}
