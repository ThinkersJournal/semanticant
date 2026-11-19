use std::collections::HashMap;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

/// Case-folding rules. The default is language-neutral Unicode folding; language-specific tailoring is
/// DATA the caller (or a data pack) supplies through [`FoldRules::with_override`], not code in the engine.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FoldRules {
    overrides: HashMap<char, String>,
    strip_marks: bool,
}

impl FoldRules {
    /// Language-neutral folding.
    pub fn new() -> Self {
        Self::default()
    }
    /// Replace `from` by `to` (applied before the default fold), e.g. Turkish `İ` -> `i`, `I` -> `ı`.
    pub fn with_override(mut self, from: char, to: &str) -> Self {
        self.overrides.insert(from, to.to_string());
        self
    }
    /// Also drop combining marks (decomposes to NFD first, result is NFC).
    pub fn strip_marks(mut self, yes: bool) -> Self {
        self.strip_marks = yes;
        self
    }
}

/// Fold `text` for caseless comparison: lowercase plus the full-folding extras that `to_lowercase` lacks
/// (`ß`->`ss`, final sigma `ς`->`σ`, long s `ſ`->`s`), after any caller overrides.
pub fn fold(text: &str, rules: &FoldRules) -> String {
    let mut out = String::with_capacity(text.len());
    let src: Box<dyn Iterator<Item = char>> = if rules.strip_marks {
        Box::new(text.nfd().filter(|c| !is_combining_mark(*c)))
    } else {
        Box::new(text.chars())
    };
    for c in src {
        if let Some(r) = rules.overrides.get(&c) {
            out.push_str(r);
            continue;
        }
        match c {
            'ß' | 'ẞ' => out.push_str("ss"),
            'ς' => out.push('σ'),
            'ſ' => out.push('s'),
            _ => out.extend(c.to_lowercase()),
        }
    }
    if rules.strip_marks {
        out.nfc().collect()
    } else {
        out
    }
}
