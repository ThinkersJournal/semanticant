use semanticant_core::{LangTag, Reason, Span, Verdict};
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;

/// Coarse class of a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Contains a letter (or ideograph).
    Word,
    /// Numeric (digits, possibly with separators as UAX #29 keeps them).
    Number,
    /// Whitespace.
    Whitespace,
    /// ASCII, General Punctuation or CJK punctuation.
    Punct,
    /// Anything else (symbols, emoji, ...).
    Symbol,
}

/// A token: its span in the original text, its text, its kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Byte range in the original text.
    pub span: Span,
    /// The token text.
    pub text: String,
    /// Its class.
    pub kind: TokenKind,
}

/// Splits text into tokens and sentences. `lang` is passed so language-aware segmenters can use it;
/// the default [`UaxSegmenter`] is language-neutral.
pub trait Segmenter {
    /// All tokens, including whitespace, covering the text exactly.
    fn tokens(&self, text: &str, lang: &LangTag) -> Vec<Token>;
    /// Sentence spans (without surrounding whitespace), each with the reasons for its edges.
    fn sentences(&self, text: &str, lang: &LangTag) -> Vec<Verdict<Span>>;
}

/// Sentence-final abbreviations that do NOT end a sentence ("Dr", "e.g"), stored case-insensitively
/// without the final dot. Data: build it from a pack for each language.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AbbreviationSet(HashSet<String>);

impl AbbreviationSet {
    /// An empty set (pure UAX #29 behaviour).
    pub fn new() -> Self {
        Self::default()
    }
    /// Build from entries like `"Dr"` or `"e.g"` (a trailing `.` is ignored).
    pub fn from_entries<I, S>(entries: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self(
            entries
                .into_iter()
                .map(|e| e.as_ref().trim_end_matches('.').to_lowercase())
                .filter(|e| !e.is_empty())
                .collect(),
        )
    }
    fn contains(&self, w: &str) -> bool {
        self.0.contains(&w.to_lowercase())
    }
}

/// UAX #29 word and sentence segmentation plus an abbreviation merge pass.
#[derive(Debug, Clone, Default)]
pub struct UaxSegmenter {
    abbreviations: AbbreviationSet,
}

impl UaxSegmenter {
    /// Pure UAX #29.
    pub fn new() -> Self {
        Self::default()
    }
    /// With abbreviations that must not end a sentence.
    pub fn with_abbreviations(abbreviations: AbbreviationSet) -> Self {
        Self { abbreviations }
    }
}

fn classify(s: &str) -> TokenKind {
    if s.chars().all(char::is_whitespace) {
        TokenKind::Whitespace
    } else if s.chars().any(char::is_alphabetic) {
        TokenKind::Word
    } else if s.chars().any(char::is_numeric) {
        TokenKind::Number
    } else if s.chars().all(|c| {
        c.is_ascii_punctuation() || matches!(c, '\u{2000}'..='\u{206F}' | '\u{3000}'..='\u{303F}')
    }) {
        TokenKind::Punct
    } else {
        TokenKind::Symbol
    }
}

impl Segmenter for UaxSegmenter {
    fn tokens(&self, text: &str, _lang: &LangTag) -> Vec<Token> {
        text.split_word_bound_indices()
            .map(|(i, w)| Token {
                span: Span::new(i, i + w.len()).expect("ordered"),
                text: w.to_string(),
                kind: classify(w),
            })
            .collect()
    }

    fn sentences(&self, text: &str, _lang: &LangTag) -> Vec<Verdict<Span>> {
        // Trimmed UAX #29 sentences, merged across a break that follows a listed abbreviation.
        let raw: Vec<(usize, usize)> = text
            .split_sentence_bound_indices()
            .filter_map(|(i, s)| {
                let t = s.trim();
                if t.is_empty() {
                    return None;
                }
                let lead = s.len() - s.trim_start().len();
                Some((i + lead, i + lead + t.len()))
            })
            .collect();
        let mut out: Vec<Verdict<Span>> = Vec::new();
        for (start, end) in raw {
            if let Some(prev) = out.last_mut() {
                let prev_span = prev.value;
                if let Some(abbr) = trailing_abbreviation(text, prev_span, &self.abbreviations) {
                    let merged = Span::new(prev_span.start(), end).expect("ordered");
                    prev.value = merged;
                    prev.reasons.retain(|r| r.rule != "text.sentence.uax29");
                    prev.reasons.push(Reason::at(
                        "text.sentence.abbreviation",
                        abbr,
                        "break after a listed abbreviation removed",
                    ));
                    continue;
                }
            }
            out.push(
                Verdict::new(Span::new(start, end).expect("ordered")).because(Reason::new(
                    "text.sentence.uax29",
                    "UAX #29 sentence boundary",
                )),
            );
        }
        out
    }
}

/// If `sentence` ends in `<abbreviation>.`, the span of that abbreviation (without the dot).
fn trailing_abbreviation(text: &str, sentence: Span, set: &AbbreviationSet) -> Option<Span> {
    let s = sentence.slice(text)?;
    let body = s.strip_suffix('.')?;
    // The abbreviation is the run of non-whitespace before the dot (so "e.g" stays whole).
    let start_in_body = body.rfind(char::is_whitespace).map_or(0, |i| {
        i + body[i..].chars().next().map_or(1, char::len_utf8)
    });
    let word = &body[start_in_body..];
    if !word.is_empty() && set.contains(word) {
        Span::new(
            sentence.start() + start_in_body,
            sentence.start() + body.len(),
        )
        .ok()
    } else {
        None
    }
}
