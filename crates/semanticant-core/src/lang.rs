use crate::CoreError;
use std::fmt;

/// A syntactically validated BCP 47 language tag (`en`, `en-GB`, `zh-Hant-TW`, `und`).
///
/// Validation is structural (subtag lengths and characters), not a registry lookup. Stored in
/// canonical case: language lower, script Title, region UPPER, the rest lower. There is no default:
/// callers must say which language their text is in, or pass [`LangTag::UNDETERMINED`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LangTag(String);

impl LangTag {
    /// The BCP 47 tag for "language not determined".
    pub const UNDETERMINED: &'static str = "und";

    /// Parse and canonicalise a tag.
    pub fn parse(input: &str) -> Result<Self, CoreError> {
        let bad = || CoreError::InvalidLangTag(input.to_string());
        let mut parts = input.split('-');
        let lang = parts.next().ok_or_else(bad)?;
        let ok_lang = (2..=3).contains(&lang.len()) || (5..=8).contains(&lang.len());
        if !ok_lang || !lang.bytes().all(|b| b.is_ascii_alphabetic()) {
            return Err(bad());
        }
        let mut out = lang.to_ascii_lowercase();
        for (i, p) in parts.enumerate() {
            if p.is_empty() || p.len() > 8 || !p.bytes().all(|b| b.is_ascii_alphanumeric()) {
                return Err(bad());
            }
            out.push('-');
            let alpha = p.bytes().all(|b| b.is_ascii_alphabetic());
            if i == 0 && p.len() == 4 && alpha {
                out.push_str(&p[..1].to_ascii_uppercase());
                out.push_str(&p[1..].to_ascii_lowercase());
            } else if p.len() == 2 && alpha {
                out.push_str(&p.to_ascii_uppercase());
            } else {
                out.push_str(&p.to_ascii_lowercase());
            }
        }
        Ok(Self(out))
    }

    /// The canonical tag text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The primary language subtag (`en` for `en-GB`).
    pub fn language(&self) -> &str {
        self.0.split('-').next().unwrap_or("")
    }
}

impl fmt::Display for LangTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for LangTag {
    type Err = CoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
