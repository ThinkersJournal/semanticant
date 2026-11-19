use crate::Measure;
use semanticant_core::{Reason, Verdict};
use std::collections::BTreeSet;

/// A set of tokens (the caller tokenises and folds; see `semanticant-text`).
type Set = BTreeSet<String>;

fn counts(a: &Set, b: &Set) -> (usize, usize, usize) {
    let inter = a.intersection(b).count();
    (inter, a.len(), b.len())
}

/// `|A ∩ B| / |A ∪ B|`; two empty sets score 1.
#[derive(Debug, Clone, Copy, Default)]
pub struct Jaccard;
/// `|A ∩ B| / min(|A|, |B|)`; any empty set scores 0 (both empty: 1).
#[derive(Debug, Clone, Copy, Default)]
pub struct Overlap;
/// `|A ∩ B| / |A|`: how much of the FIRST set is contained in the second (not symmetric); empty first set scores 1.
#[derive(Debug, Clone, Copy, Default)]
pub struct Containment;

impl Measure<Set> for Jaccard {
    fn score(&self, a: &Set, b: &Set) -> Verdict<f64> {
        let (i, la, lb) = counts(a, b);
        let u = la + lb - i;
        let s = if u == 0 { 1.0 } else { i as f64 / u as f64 };
        Verdict::new(s).because(Reason::new(
            "lexicon.tokens.jaccard",
            format!("{i} shared of {u} distinct"),
        ))
    }
}
impl Measure<Set> for Overlap {
    fn score(&self, a: &Set, b: &Set) -> Verdict<f64> {
        let (i, la, lb) = counts(a, b);
        let m = la.min(lb);
        let s = if la == 0 && lb == 0 {
            1.0
        } else if m == 0 {
            0.0
        } else {
            i as f64 / m as f64
        };
        Verdict::new(s).because(Reason::new(
            "lexicon.tokens.overlap",
            format!("{i} shared, smaller set has {m}"),
        ))
    }
}
impl Measure<Set> for Containment {
    fn score(&self, a: &Set, b: &Set) -> Verdict<f64> {
        let (i, la, _) = counts(a, b);
        let s = if la == 0 { 1.0 } else { i as f64 / la as f64 };
        Verdict::new(s).because(Reason::new(
            "lexicon.tokens.containment",
            format!("{i} of the first set's {la} tokens found in the second"),
        ))
    }
}
