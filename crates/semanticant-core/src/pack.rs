//! The data-pack manifest: the TYPES and their validation. Reading `pack.toml`, hashing files and
//! loading data live in a later crate; this module holds only what siblings must agree on.
//!
//! A pack is a directory containing `pack.toml` (a [`Manifest`]) and the data files it lists, each
//! with a SHA-256 checksum. Data files are text (JSON Lines or TSV) in step 1.

use crate::{CoreError, LangTag, Provenance};
use std::collections::HashSet;

/// The manifest schema version this crate understands.
pub const SCHEMA_VERSION: u32 = 1;
/// The manifest file name inside a pack directory.
pub const MANIFEST_FILE: &str = "pack.toml";

/// A lowercase hex SHA-256 digest (64 characters).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sha256Hex(String);

impl Sha256Hex {
    /// Accept exactly 64 lowercase hex digits.
    pub fn parse(s: &str) -> Result<Self, CoreError> {
        if s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            Ok(Self(s.to_string()))
        } else {
            Err(CoreError::InvalidChecksum(s.to_string()))
        }
    }
    /// The digest text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One data file listed by a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// Relative path inside the pack, `/`-separated, no `..`, not absolute.
    pub name: String,
    /// Expected SHA-256 of the file's bytes.
    pub sha256: Sha256Hex,
}

/// What a pack contains, free-form but non-empty (e.g. `lexicon`, `abbreviations`, `number-words`).
pub type PackKind = String;

/// A data pack's manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Pack name.
    pub name: String,
    /// Pack version (its own, not the crate's).
    pub version: String,
    /// What kind of data it holds.
    pub kind: PackKind,
    /// Manifest schema version; must equal [`SCHEMA_VERSION`].
    pub schema: u32,
    /// Language of the data, if it is language-specific.
    pub lang: Option<LangTag>,
    /// Where it came from and the terms of use. Required.
    pub provenance: Provenance,
    /// The data files and their checksums.
    pub files: Vec<FileEntry>,
}

impl Manifest {
    /// Refuse a manifest that is incomplete or unsafe: unknown schema, empty name/version/kind,
    /// incomplete provenance (no licence or source), no files, duplicate files, or file names that
    /// are empty, absolute, contain `..`, backslashes or NUL.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.schema != SCHEMA_VERSION {
            return Err(CoreError::UnsupportedSchema(self.schema));
        }
        for (n, v) in [
            ("name", &self.name),
            ("version", &self.version),
            ("kind", &self.kind),
        ] {
            if v.trim().is_empty() {
                return Err(CoreError::EmptyField(n));
            }
        }
        self.provenance.validate()?;
        if self.files.is_empty() {
            return Err(CoreError::EmptyField("files"));
        }
        let mut seen = HashSet::new();
        for f in &self.files {
            check_file_name(&f.name)?;
            if !seen.insert(f.name.as_str()) {
                return Err(CoreError::DuplicateFile(f.name.clone()));
            }
        }
        Ok(())
    }
}

fn check_file_name(name: &str) -> Result<(), CoreError> {
    let bad = || CoreError::InvalidFileName(name.to_string());
    if name.is_empty()
        || name.starts_with('/')
        || name.contains('\\')
        || name.contains('\0')
        || name.contains(':')
    {
        return Err(bad());
    }
    if name
        .split('/')
        .any(|seg| seg.is_empty() || seg == "." || seg == "..")
    {
        return Err(bad());
    }
    Ok(())
}
