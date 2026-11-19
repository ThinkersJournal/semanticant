use crate::Measure;
use semanticant_core::{Reason, Verdict};

/// Turns text into a vector. SUPPLIED BY THE CALLER; this project ships no model.
pub trait Embedder {
    /// Embed `text`. Vectors from one embedder should have one length.
    fn embed(&self, text: &str) -> Vec<f32>;
}

/// Cosine similarity of two texts under a caller-supplied [`Embedder`], clamped to `[0, 1]`
/// (negative cosines score 0). A zero vector or a length mismatch scores 0 with a reason saying so.
#[derive(Debug, Clone)]
pub struct Cosine<E>(pub E);

impl<E: Embedder> Measure<str> for Cosine<E> {
    fn score(&self, a: &str, b: &str) -> Verdict<f64> {
        let (va, vb) = (self.0.embed(a), self.0.embed(b));
        if va.len() != vb.len() {
            return Verdict::new(0.0).because(Reason::new(
                "lexicon.cosine.dimension_mismatch",
                format!("vector lengths {} and {}", va.len(), vb.len()),
            ));
        }
        let dot: f64 = va
            .iter()
            .zip(&vb)
            .map(|(x, y)| f64::from(*x) * f64::from(*y))
            .sum();
        let na: f64 = va.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>().sqrt();
        let nb: f64 = vb.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>().sqrt();
        if na == 0.0 || nb == 0.0 {
            return Verdict::new(0.0).because(Reason::new(
                "lexicon.cosine.zero_vector",
                "one of the vectors has zero length",
            ));
        }
        let c = dot / (na * nb);
        Verdict::new(c.clamp(0.0, 1.0))
            .because(Reason::new("lexicon.cosine", format!("cosine {c:.4}")))
    }
}
