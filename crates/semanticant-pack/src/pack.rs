use crate::{PackError, parse_manifest};
use semanticant_core::pack::{MANIFEST_FILE, Manifest};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

/// Lowercase hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A loaded, verified pack: manifest plus the bytes of every listed file.
#[derive(Debug, Clone)]
pub struct Pack {
    manifest: Manifest,
    files: HashMap<String, Vec<u8>>,
}

impl Pack {
    /// Load a pack directory: parse and validate `pack.toml`, read every listed file, verify each SHA-256.
    /// Nothing is returned unless everything checks out.
    pub fn open(dir: &Path) -> Result<Pack, PackError> {
        let mpath = dir.join(MANIFEST_FILE);
        let text = std::fs::read_to_string(&mpath).map_err(|source| PackError::Io {
            path: mpath,
            source,
        })?;
        let manifest = parse_manifest(&text)?;
        let mut files = HashMap::new();
        for entry in &manifest.files {
            let path = dir.join(&entry.name);
            let meta = std::fs::symlink_metadata(&path).map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => PackError::Missing(entry.name.clone()),
                _ => PackError::Io {
                    path: path.clone(),
                    source: e,
                },
            })?;
            if meta.file_type().is_symlink() {
                return Err(PackError::Symlink(entry.name.clone()));
            }
            let bytes = std::fs::read(&path).map_err(|source| PackError::Io { path, source })?;
            files.insert(entry.name.clone(), bytes);
        }
        Self::from_parts(manifest, files)
    }

    /// Build a pack from an already-parsed manifest and in-memory files (tests, generated data).
    /// Applies the same validation and checksum verification as [`Pack::open`].
    pub fn from_parts(
        manifest: Manifest,
        files: HashMap<String, Vec<u8>>,
    ) -> Result<Pack, PackError> {
        manifest.validate()?;
        for entry in &manifest.files {
            let bytes = files
                .get(&entry.name)
                .ok_or_else(|| PackError::Missing(entry.name.clone()))?;
            let actual = sha256_hex(bytes);
            if actual != entry.sha256.as_str() {
                return Err(PackError::ChecksumMismatch {
                    file: entry.name.clone(),
                    expected: entry.sha256.as_str().to_string(),
                    actual,
                });
            }
        }
        // Only listed files are kept.
        let files = files
            .into_iter()
            .filter(|(k, _)| manifest.files.iter().any(|e| &e.name == k))
            .collect();
        Ok(Pack { manifest, files })
    }

    /// The validated manifest.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    /// The verified bytes of a listed file.
    pub fn file(&self, name: &str) -> Option<&[u8]> {
        self.files.get(name).map(Vec::as_slice)
    }
    /// A listed file as UTF-8 text (`None` if absent or not valid UTF-8).
    pub fn text(&self, name: &str) -> Option<&str> {
        self.file(name).and_then(|b| std::str::from_utf8(b).ok())
    }
    /// A listed text file as rows of tab-separated fields; blank lines and lines starting with `#` are skipped.
    pub fn tsv(&self, name: &str) -> Option<Vec<Vec<&str>>> {
        let t = self.text(name)?;
        Some(
            t.lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .map(|l| l.split('\t').collect())
                .collect(),
        )
    }
}
