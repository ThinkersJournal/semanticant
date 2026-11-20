use crate::ValueError;
use rust_decimal::Decimal;
use semanticant_core::{Reason, Span, Verdict};
use semanticant_pack::Pack;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

/// A value found at a span of the original text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found<T> {
    /// Byte range in the original text.
    pub span: Span,
    /// The parsed value.
    pub value: T,
}

/// What a number word contributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordKind {
    /// 0-19 ("zero" .. "nineteen").
    Unit(u32),
    /// 20, 30 .. 90.
    Tens(u32),
    /// A multiplier: 100, 1000, 1_000_000 ...
    Scale(u64),
}

/// How numbers are written in a language: separators and number words. DATA, supplied by the caller or a pack.
#[derive(Debug, Clone)]
pub struct NumberRules {
    decimal: char,
    groups: Vec<char>,
    words: HashMap<String, WordKind>,
    connectors: HashSet<String>,
}

impl NumberRules {
    /// Rules with the given decimal separator and thousands-group separators and no number words yet.
    pub fn new(decimal: char, groups: &[char]) -> Self {
        Self {
            decimal,
            groups: groups.to_vec(),
            words: HashMap::new(),
            connectors: HashSet::new(),
        }
    }
    /// Add a number word (stored lowercase).
    pub fn with_word(mut self, word: &str, kind: WordKind) -> Self {
        self.words.insert(word.to_lowercase(), kind);
        self
    }
    /// Add a connector that may sit between number words ("and" in "one hundred and five").
    pub fn with_connector(mut self, word: &str) -> Self {
        self.connectors.insert(word.to_lowercase());
        self
    }
    /// Build the words from a pack of `kind = "number-words"` with `words.tsv` rows `word<TAB>unit|tens|scale|connector<TAB>value`
    /// (value ignored for `connector`). Separators are passed in because they belong to the locale, not the word list.
    pub fn from_pack(pack: &Pack, decimal: char, groups: &[char]) -> Result<Self, ValueError> {
        let kind = &pack.manifest().kind;
        if kind != "number-words" {
            return Err(ValueError::WrongKind {
                found: kind.clone(),
                want: "number-words",
            });
        }
        let rows = pack
            .tsv("words.tsv")
            .ok_or(ValueError::MissingFile("words.tsv"))?;
        let mut r = Self::new(decimal, groups);
        for (n, row) in rows.iter().enumerate() {
            let n = n + 1;
            if row.len() < 2 {
                return Err(ValueError::BadRow(
                    "words.tsv",
                    n,
                    "want word<TAB>kind[<TAB>value]",
                ));
            }
            if row[1] == "connector" {
                r = r.with_connector(row[0]);
                continue;
            }
            let v: u64 =
                row.get(2)
                    .and_then(|v| v.trim().parse().ok())
                    .ok_or(ValueError::BadRow(
                        "words.tsv",
                        n,
                        "value is not a whole number",
                    ))?;
            let kind = match row[1] {
                "unit" if v < 20 => WordKind::Unit(v as u32),
                "tens" if (20..100).contains(&v) => WordKind::Tens(v as u32),
                "scale" if v >= 100 => WordKind::Scale(v),
                _ => {
                    return Err(ValueError::BadRow(
                        "words.tsv",
                        n,
                        "kind/value mismatch (unit 0-19, tens 20-99, scale >= 100)",
                    ));
                }
            };
            r = r.with_word(row[0], kind);
        }
        Ok(r)
    }
}

/// A digit-number found in text, before filtering. `attached` = a letter follows immediately ("5km", "3rd").
pub(crate) struct Raw {
    pub span: Span,
    pub value: Decimal,
    pub note: String,
    pub attached: bool,
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

pub(crate) fn scan_digits(text: &str, rules: &NumberRules) -> Vec<Raw> {
    let cs: Vec<(usize, char)> = text.char_indices().collect();
    let at = |i: usize| cs.get(i).map(|x| x.1);
    let byte = |i: usize| cs.get(i).map_or(text.len(), |x| x.0);
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        if !is_digit(cs[i].1) || (i > 0 && cs[i - 1].1.is_alphanumeric()) {
            i += 1;
            continue;
        }
        let mut start = i;
        let mut negative = false;
        if i > 0
            && matches!(cs[i - 1].1, '-' | '+' | '\u{2212}')
            && (i < 2 || !cs[i - 2].1.is_alphanumeric())
        {
            start = i - 1;
            negative = cs[i - 1].1 != '+';
        }
        let mut digits = String::new();
        let mut j = i;
        while at(j).is_some_and(is_digit) {
            digits.push(cs[j].1);
            j += 1;
        }
        let first_run = j - i;
        let mut grouped = false;
        // thousands groups: sep + exactly three digits, and the first run is 1-3 digits
        while first_run <= 3
            && at(j).is_some_and(|c| rules.groups.contains(&c))
            && (1..=3).all(|k| at(j + k).is_some_and(is_digit))
            && !at(j + 4).is_some_and(is_digit)
        {
            for k in 1..=3 {
                digits.push(cs[j + k].1);
            }
            j += 4;
            grouped = true;
        }
        let mut fraction = false;
        if at(j) == Some(rules.decimal) && at(j + 1).is_some_and(is_digit) {
            digits.push('.');
            j += 1;
            while at(j).is_some_and(is_digit) {
                digits.push(cs[j].1);
                j += 1;
            }
            fraction = true;
        }
        if let Ok(mut v) = Decimal::from_str(&digits) {
            if negative {
                v.set_sign_negative(true);
            }
            let span = Span::new(byte(start), byte(j)).expect("ordered");
            let note = format!(
                "digits{}{}",
                if grouped {
                    " with group separators"
                } else {
                    ""
                },
                if fraction { " and a decimal part" } else { "" }
            );
            out.push(Raw {
                span,
                value: v,
                note,
                attached: at(j).is_some_and(char::is_alphabetic),
            });
        }
        i = j.max(i + 1);
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Last {
    Unit,
    Tens,
    Scale,
}

pub(crate) fn scan_words(text: &str, rules: &NumberRules) -> Vec<(Span, Decimal, String)> {
    // alphabetic runs, joined into chains by single spaces / hyphens
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut cur: Option<usize> = None;
    for (i, c) in text.char_indices() {
        match (c.is_alphabetic(), cur) {
            (true, None) => cur = Some(i),
            (false, Some(s)) => {
                runs.push((s, i));
                cur = None;
            }
            _ => {}
        }
    }
    if let Some(s) = cur {
        runs.push((s, text.len()));
    }
    let word = |r: &(usize, usize)| text[r.0..r.1].to_lowercase();
    let joined =
        |a: &(usize, usize), b: &(usize, usize)| matches!(&text[a.1..b.0], " " | "-" | "\u{a0}");
    let mut out = Vec::new();
    let mut k = 0;
    while k < runs.len() {
        let Some(&first_kind) = rules.words.get(&word(&runs[k])) else {
            k += 1;
            continue;
        };
        // build one number starting at run k
        let (mut total, mut current, mut last_scale) = (0u128, 0u128, u128::MAX);
        let mut last: Option<Last> = None;
        let start = k;
        let mut end = k;
        let mut m = k;
        let mut kind = first_kind;
        loop {
            let ok = match (kind, last) {
                (WordKind::Unit(_), Some(Last::Unit)) => false,
                (WordKind::Unit(v), Some(Last::Tens)) => v < 10 && v > 0,
                (WordKind::Tens(_), Some(Last::Unit | Last::Tens)) => false,
                (WordKind::Scale(s), _) => {
                    (s == 100 && current < 100) || (s != 100 && u128::from(s) < last_scale)
                }
                _ => true,
            };
            if !ok {
                break;
            }
            match kind {
                WordKind::Unit(v) => {
                    current += u128::from(v);
                    last = Some(Last::Unit);
                }
                WordKind::Tens(v) => {
                    current += u128::from(v);
                    last = Some(Last::Tens);
                }
                WordKind::Scale(s) => {
                    let s = u128::from(s);
                    let Some(prod) = current.max(1).checked_mul(s) else {
                        break;
                    };
                    if s == 100 {
                        current = prod;
                    } else {
                        let Some(t) = total.checked_add(prod) else {
                            break;
                        };
                        total = t;
                        current = 0;
                        last_scale = s;
                    }
                    last = Some(Last::Scale);
                }
            }
            end = m;
            // look ahead: a number word, or a connector followed by a number word
            let mut n = m + 1;
            if n < runs.len()
                && joined(&runs[m], &runs[n])
                && rules.connectors.contains(&word(&runs[n]))
                && n + 1 < runs.len()
                && joined(&runs[n], &runs[n + 1])
            {
                n += 1;
            }
            match runs
                .get(n)
                .filter(|r| joined(&runs[n - 1], r))
                .and_then(|r| rules.words.get(&word(r)).copied())
            {
                Some(next) => {
                    kind = next;
                    m = n;
                }
                None => break,
            }
        }
        let Some(value) = total.checked_add(current) else {
            k = end + 1;
            continue;
        };
        if let Ok(v) = i128::try_from(value) {
            let span = Span::new(runs[start].0, runs[end].1).expect("ordered");
            out.push((
                span,
                Decimal::from(v),
                format!("{} number word(s)", end - start + 1),
            ));
        }
        k = end + 1;
    }
    out
}

/// Numbers in `text`: digit forms (sign, group separators, decimal part) and spelled-out numbers from the rules' word list.
/// A digit run directly followed by a letter ("3rd", "5km") is NOT returned here (see `parse_quantities`).
/// Numbers beyond `Decimal`'s range (about 28 digits) are skipped. Results are in text order.
pub fn parse_numbers(text: &str, rules: &NumberRules) -> Vec<Verdict<Found<Decimal>>> {
    let mut all: Vec<Verdict<Found<Decimal>>> = Vec::new();
    for r in scan_digits(text, rules).into_iter().filter(|r| !r.attached) {
        all.push(
            Verdict::new(Found {
                span: r.span,
                value: r.value,
            })
            .because(Reason::at("value.number.digits", r.span, r.note)),
        );
    }
    for (span, v, note) in scan_words(text, rules) {
        all.push(Verdict::new(Found { span, value: v }).because(Reason::at(
            "value.number.words",
            span,
            note,
        )));
    }
    all.sort_by_key(|v| v.value.span);
    all
}
