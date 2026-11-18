use crate::Measure;
use semanticant_core::{Reason, Verdict};

fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

/// Normalised Levenshtein similarity over Unicode scalar values: `1 - distance / max_len` (two empty strings score 1).
#[derive(Debug, Clone, Copy, Default)]
pub struct Levenshtein;

impl Levenshtein {
    /// The edit distance (insert, delete, substitute) between two strings.
    pub fn distance(a: &str, b: &str) -> usize {
        let (a, b) = (chars(a), chars(b));
        let mut prev: Vec<usize> = (0..=b.len()).collect();
        for (i, ca) in a.iter().enumerate() {
            let mut cur = vec![i + 1];
            for (j, cb) in b.iter().enumerate() {
                cur.push(
                    (prev[j] + usize::from(ca != cb))
                        .min(prev[j + 1] + 1)
                        .min(cur[j] + 1),
                );
            }
            prev = cur;
        }
        prev[b.len()]
    }
}

impl Measure<str> for Levenshtein {
    fn score(&self, a: &str, b: &str) -> Verdict<f64> {
        let max = a.chars().count().max(b.chars().count());
        let d = Self::distance(a, b);
        let s = if max == 0 {
            1.0
        } else {
            1.0 - d as f64 / max as f64
        };
        Verdict::new(s).because(Reason::new(
            "lexicon.string.levenshtein",
            format!("edit distance {d} over max length {max}"),
        ))
    }
}

/// Jaro-Winkler similarity (prefix scale 0.1, prefix cap 4, boost only when Jaro >= 0.7).
#[derive(Debug, Clone, Copy, Default)]
pub struct JaroWinkler;

fn jaro(a: &[char], b: &[char]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let window = (a.len().max(b.len()) / 2).saturating_sub(1);
    let (mut am, mut bm) = (vec![false; a.len()], vec![false; b.len()]);
    let mut matches = 0usize;
    for i in 0..a.len() {
        let lo = i.saturating_sub(window);
        let hi = (i + window + 1).min(b.len());
        for j in lo..hi {
            if !bm[j] && a[i] == b[j] {
                am[i] = true;
                bm[j] = true;
                matches += 1;
                break;
            }
        }
    }
    if matches == 0 {
        return 0.0;
    }
    let (mut k, mut half_transpositions) = (0usize, 0usize);
    for i in 0..a.len() {
        if am[i] {
            while !bm[k] {
                k += 1;
            }
            if a[i] != b[k] {
                half_transpositions += 1;
            }
            k += 1;
        }
    }
    let m = matches as f64;
    (m / a.len() as f64 + m / b.len() as f64 + (m - half_transpositions as f64 / 2.0) / m) / 3.0
}

impl Measure<str> for JaroWinkler {
    fn score(&self, a: &str, b: &str) -> Verdict<f64> {
        let (ca, cb) = (chars(a), chars(b));
        let j = jaro(&ca, &cb);
        let prefix = ca
            .iter()
            .zip(&cb)
            .take(4)
            .take_while(|(x, y)| x == y)
            .count();
        let s = if j >= 0.7 {
            j + prefix as f64 * 0.1 * (1.0 - j)
        } else {
            j
        };
        Verdict::new(s).because(Reason::new(
            "lexicon.string.jaro_winkler",
            format!("jaro {j:.4}, common prefix {prefix}"),
        ))
    }
}

/// Dice coefficient over character bigrams (strings shorter than 2 chars compare by equality).
#[derive(Debug, Clone, Copy, Default)]
pub struct Dice;

impl Measure<str> for Dice {
    fn score(&self, a: &str, b: &str) -> Verdict<f64> {
        let grams = |s: &str| -> Vec<(char, char)> {
            let c = chars(s);
            c.windows(2).map(|w| (w[0], w[1])).collect()
        };
        let (ga, gb) = (grams(a), grams(b));
        let (s, note) = if ga.is_empty() || gb.is_empty() {
            (
                if a == b { 1.0 } else { 0.0 },
                "a string under 2 chars: compared by equality".to_string(),
            )
        } else {
            let mut rest = gb.clone();
            let mut common = 0usize;
            for g in &ga {
                if let Some(p) = rest.iter().position(|x| x == g) {
                    rest.swap_remove(p);
                    common += 1;
                }
            }
            (
                2.0 * common as f64 / (ga.len() + gb.len()) as f64,
                format!("{common} shared bigrams of {} and {}", ga.len(), gb.len()),
            )
        };
        Verdict::new(s).because(Reason::new("lexicon.string.dice", note))
    }
}
