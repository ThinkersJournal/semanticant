use crate::{Measure, Relation, SenseId, SenseInventory};
use semanticant_core::{Reason, Verdict};
use std::collections::{HashMap, HashSet, VecDeque};

/// Distances from `start` up the hypernym edges (itself at 0). Cycle-safe.
fn ancestors(inv: &dyn SenseInventory, start: SenseId) -> HashMap<SenseId, usize> {
    let mut dist = HashMap::from([(start, 0usize)]);
    let mut q = VecDeque::from([start]);
    while let Some(s) = q.pop_front() {
        let d = dist[&s];
        for p in inv.relations(s, &Relation::Hypernym) {
            if let std::collections::hash_map::Entry::Vacant(e) = dist.entry(p) {
                e.insert(d + 1);
                q.push_back(p);
            }
        }
    }
    dist
}

/// Depth of a sense: a root (no hypernyms) has depth 1; otherwise one more than the SHALLOWEST path to a root.
fn depth(inv: &dyn SenseInventory, s: SenseId) -> usize {
    let anc = ancestors(inv, s);
    anc.iter()
        .filter(|(a, _)| inv.relations(**a, &Relation::Hypernym).is_empty())
        .map(|(_, d)| d + 1)
        .min()
        .unwrap_or(1)
}

/// Number of strict descendants (via hyponym edges). Cycle-safe.
fn descendants(inv: &dyn SenseInventory, s: SenseId) -> usize {
    let mut seen = HashSet::from([s]);
    let mut q = VecDeque::from([s]);
    while let Some(x) = q.pop_front() {
        for c in inv.relations(x, &Relation::Hyponym) {
            if seen.insert(c) {
                q.push_back(c);
            }
        }
    }
    seen.len() - 1
}

/// Intrinsic information content (Seco et al. 2004): `1 - ln(descendants + 1) / ln(N)`, from the taxonomy alone
/// (no corpus). A leaf has 1; the top of a taxonomy with every sense below it has 0. `N <= 1` gives 0.
fn intrinsic_ic(inv: &dyn SenseInventory, s: SenseId, n: usize) -> f64 {
    if n <= 1 {
        return 0.0;
    }
    1.0 - ((descendants(inv, s) + 1) as f64).ln() / (n as f64).ln()
}

fn key_of(inv: &dyn SenseInventory, s: SenseId) -> String {
    inv.lemmas(s)
        .first()
        .map_or_else(|| format!("#{}", s.0), |l| (*l).to_string())
}

/// Deepest common ancestor (ties: smaller summed distance, then lower id).
fn common(inv: &dyn SenseInventory, a: SenseId, b: SenseId) -> Option<SenseId> {
    let (aa, ab) = (ancestors(inv, a), ancestors(inv, b));
    let mut best: Option<(usize, usize, SenseId)> = None;
    for (s, da) in &aa {
        if let Some(db) = ab.get(s) {
            let dep = depth(inv, *s);
            let total = da + db;
            let better = best.as_ref().is_none_or(|(bd, bt, bs)| {
                dep > *bd || (dep == *bd && (total < *bt || (total == *bt && s.0 < bs.0)))
            });
            if better {
                best = Some((dep, total, *s));
            }
        }
    }
    best.map(|(_, _, s)| s)
}

fn none(rule: &'static str, inv: &dyn SenseInventory, a: SenseId, b: SenseId) -> Verdict<f64> {
    Verdict::new(0.0).because(Reason::new(
        rule,
        format!(
            "{} and {} share no ancestor",
            key_of(inv, a),
            key_of(inv, b)
        ),
    ))
}

/// `1 / (1 + d)` where `d` is the shortest path between the senses through a common ancestor (edges = hypernym links).
pub struct PathSimilarity<'a>(pub &'a dyn SenseInventory);
/// Wu-Palmer: `2 * depth(lcs) / (depth(a) + depth(b))` with the deepest common ancestor as `lcs`; depth counts a root as 1.
pub struct WuPalmer<'a>(pub &'a dyn SenseInventory);
/// Resnik: intrinsic information content of the most informative common ancestor. Range `[0, 1]` with intrinsic IC.
pub struct Resnik<'a>(pub &'a dyn SenseInventory);
/// Lin: `2 * IC(lcs) / (IC(a) + IC(b))` with intrinsic IC; two zero-IC senses score 1 if equal, else 0.
pub struct Lin<'a>(pub &'a dyn SenseInventory);

impl Measure<SenseId> for PathSimilarity<'_> {
    fn score(&self, a: &SenseId, b: &SenseId) -> Verdict<f64> {
        let inv = self.0;
        let (aa, ab) = (ancestors(inv, *a), ancestors(inv, *b));
        let d = aa
            .iter()
            .filter_map(|(s, da)| ab.get(s).map(|db| da + db))
            .min();
        match d {
            None => none("lexicon.taxonomy.no_common_ancestor", inv, *a, *b),
            Some(d) => Verdict::new(1.0 / (1.0 + d as f64)).because(Reason::new(
                "lexicon.taxonomy.path",
                format!("shortest path {d} edges"),
            )),
        }
    }
}

impl Measure<SenseId> for WuPalmer<'_> {
    fn score(&self, a: &SenseId, b: &SenseId) -> Verdict<f64> {
        let inv = self.0;
        match common(inv, *a, *b) {
            None => none("lexicon.taxonomy.no_common_ancestor", inv, *a, *b),
            Some(lcs) => {
                let (dl, da, db) = (depth(inv, lcs), depth(inv, *a), depth(inv, *b));
                Verdict::new(2.0 * dl as f64 / (da + db) as f64).because(Reason::new(
                    "lexicon.taxonomy.wu_palmer",
                    format!(
                        "lcs {} at depth {dl}; depths {da} and {db}",
                        key_of(inv, lcs)
                    ),
                ))
            }
        }
    }
}

/// The common ancestor with the highest intrinsic IC.
fn best_ic(inv: &dyn SenseInventory, a: SenseId, b: SenseId, n: usize) -> Option<(SenseId, f64)> {
    let (aa, ab) = (ancestors(inv, a), ancestors(inv, b));
    aa.keys()
        .filter(|s| ab.contains_key(s))
        .map(|s| (*s, intrinsic_ic(inv, *s, n)))
        .max_by(|x, y| x.1.total_cmp(&y.1).then(y.0.0.cmp(&x.0.0)))
}

impl Measure<SenseId> for Resnik<'_> {
    fn score(&self, a: &SenseId, b: &SenseId) -> Verdict<f64> {
        let inv = self.0;
        let n = inv.all_senses().len();
        match best_ic(inv, *a, *b, n) {
            None => none("lexicon.taxonomy.no_common_ancestor", inv, *a, *b),
            Some((s, ic)) => Verdict::new(ic).because(Reason::new(
                "lexicon.taxonomy.resnik",
                format!(
                    "most informative ancestor {} has IC {ic:.4}",
                    key_of(inv, s)
                ),
            )),
        }
    }
}

impl Measure<SenseId> for Lin<'_> {
    fn score(&self, a: &SenseId, b: &SenseId) -> Verdict<f64> {
        let inv = self.0;
        let n = inv.all_senses().len();
        match best_ic(inv, *a, *b, n) {
            None => none("lexicon.taxonomy.no_common_ancestor", inv, *a, *b),
            Some((s, ic)) => {
                let denom = intrinsic_ic(inv, *a, n) + intrinsic_ic(inv, *b, n);
                let v = if denom == 0.0 {
                    f64::from(u8::from(a == b))
                } else {
                    (2.0 * ic / denom).min(1.0)
                };
                Verdict::new(v).because(Reason::new(
                    "lexicon.taxonomy.lin",
                    format!(
                        "ancestor {} IC {ic:.4}; IC(a)+IC(b) = {denom:.4}",
                        key_of(inv, s)
                    ),
                ))
            }
        }
    }
}
