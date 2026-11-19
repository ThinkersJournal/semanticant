use semanticant_core::Span;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

/// A Unicode normalisation form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormForm {
    /// Canonical composition.
    Nfc,
    /// Canonical decomposition.
    Nfd,
    /// Compatibility composition.
    Nfkc,
    /// Compatibility decomposition.
    Nfkd,
}

/// Normalised text plus a map from output positions back to the original text.
///
/// Normalisation is applied per grapheme cluster, so each output piece maps to exactly one original
/// cluster. For the inputs where a normalisation form never composes across a cluster boundary
/// (everything but pathological sequences) the result equals whole-string normalisation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalised {
    /// The normalised text.
    pub text: String,
    // (start of this piece in `text`, original span of the cluster it came from)
    pieces: Vec<(usize, Span)>,
}

impl Normalised {
    /// The original-text span that produced the given span of the normalised text: the union of every
    /// cluster the span touches. `None` if the span is out of range or empty text is addressed.
    pub fn original_span(&self, out: Span) -> Option<Span> {
        if out.end() > self.text.len() || out.is_empty() {
            return None;
        }
        let mut result: Option<Span> = None;
        for (i, (start, orig)) in self.pieces.iter().enumerate() {
            let end = self.pieces.get(i + 1).map_or(self.text.len(), |p| p.0);
            if *start < out.end() && out.start() < end {
                result = Some(result.map_or(*orig, |r| r.union(orig)));
            }
        }
        result
    }
}

/// Normalise `text` to `form`, keeping the offset map.
pub fn normalise(text: &str, form: NormForm) -> Normalised {
    let mut out = String::with_capacity(text.len());
    let mut pieces = Vec::new();
    for (idx, g) in text.grapheme_indices(true) {
        pieces.push((out.len(), Span::new(idx, idx + g.len()).expect("ordered")));
        match form {
            NormForm::Nfc => out.extend(g.nfc()),
            NormForm::Nfd => out.extend(g.nfd()),
            NormForm::Nfkc => out.extend(g.nfkc()),
            NormForm::Nfkd => out.extend(g.nfkd()),
        }
    }
    Normalised { text: out, pieces }
}
