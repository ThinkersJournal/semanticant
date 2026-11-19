use crate::numbers::{scan_digits, scan_words};
use crate::{Found, NumberRules, ValueError};
use rust_decimal::Decimal;
use semanticant_core::{Reason, Span, Verdict};
use semanticant_pack::Pack;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// What a unit is: its dimension and how to reach the dimension's base unit: `base = value * factor + offset`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitDef {
    /// Dimension name from the data ("length", "mass", "temperature", ...).
    pub dimension: String,
    /// Multiplier to the base unit; never zero.
    pub factor: Decimal,
    /// Added after multiplying (temperatures); usually zero.
    pub offset: Decimal,
}

/// A number with a unit. `unit` is the unit id from the table, not the surface text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quantity {
    /// The number.
    pub value: Decimal,
    /// Unit id (see [`UnitTable::get`]).
    pub unit: String,
}

/// Units and the surface forms that name them. DATA from a pack of `kind = "units"`.
#[derive(Debug, Clone, Default)]
pub struct UnitTable {
    units: HashMap<String, UnitDef>,
    surfaces: HashMap<String, String>,
    max_words: usize,
}

impl UnitTable {
    /// An empty table.
    pub fn new() -> Self {
        Self::default()
    }
    /// Add a unit surface form for a unit id; the first definition of an id fixes its values.
    pub fn with_unit(mut self, surface: &str, id: &str, def: UnitDef) -> Result<Self, ValueError> {
        if let Some(old) = self.units.get(id) {
            if old != &def {
                return Err(ValueError::ConflictingUnit(id.to_string()));
            }
        } else {
            self.units.insert(id.to_string(), def);
        }
        self.max_words = self.max_words.max(surface.split(' ').count());
        self.surfaces.insert(surface.to_string(), id.to_string());
        Ok(self)
    }
    /// Build from `units.tsv` rows `surface<TAB>id<TAB>dimension<TAB>factor[<TAB>offset]`.
    pub fn from_pack(pack: &Pack) -> Result<Self, ValueError> {
        let kind = &pack.manifest().kind;
        if kind != "units" {
            return Err(ValueError::WrongKind {
                found: kind.clone(),
                want: "units",
            });
        }
        let rows = pack
            .tsv("units.tsv")
            .ok_or(ValueError::MissingFile("units.tsv"))?;
        let mut t = Self::new();
        for (n, row) in rows.iter().enumerate() {
            let n = n + 1;
            if !(4..=5).contains(&row.len()) {
                return Err(ValueError::BadRow("units.tsv", n, "want 4 or 5 columns"));
            }
            let factor = Decimal::from_str(row[3].trim())
                .map_err(|_| ValueError::BadRow("units.tsv", n, "factor is not a decimal"))?;
            if factor.is_zero() {
                return Err(ValueError::BadRow("units.tsv", n, "factor is zero"));
            }
            let offset = match row.get(4) {
                Some(o) => Decimal::from_str(o.trim())
                    .map_err(|_| ValueError::BadRow("units.tsv", n, "offset is not a decimal"))?,
                None => Decimal::ZERO,
            };
            t = t.with_unit(
                row[0],
                row[1],
                UnitDef {
                    dimension: row[2].to_string(),
                    factor,
                    offset,
                },
            )?;
        }
        Ok(t)
    }
    /// The definition of a unit id.
    pub fn get(&self, id: &str) -> Option<&UnitDef> {
        self.units.get(id)
    }
}

/// Why a conversion failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvertError {
    /// A unit id is not in the table.
    UnknownUnit(String),
    /// The units measure different dimensions (from, to).
    DimensionMismatch(String, String),
    /// The result does not fit a `Decimal`.
    Overflow,
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownUnit(u) => write!(f, "unknown unit {u:?}"),
            Self::DimensionMismatch(a, b) => write!(f, "cannot convert {a} to {b}"),
            Self::Overflow => f.write_str("result out of range"),
        }
    }
}
impl std::error::Error for ConvertError {}

/// Convert `q` to unit id `to`. Exact for decimal factors; a division that does not terminate rounds at `Decimal`'s 28 digits.
pub fn convert(
    q: &Quantity,
    to: &str,
    table: &UnitTable,
) -> Result<Verdict<Decimal>, ConvertError> {
    let from = table
        .get(&q.unit)
        .ok_or_else(|| ConvertError::UnknownUnit(q.unit.clone()))?;
    let dest = table
        .get(to)
        .ok_or_else(|| ConvertError::UnknownUnit(to.to_string()))?;
    if from.dimension != dest.dimension {
        return Err(ConvertError::DimensionMismatch(
            from.dimension.clone(),
            dest.dimension.clone(),
        ));
    }
    let base = q
        .value
        .checked_mul(from.factor)
        .and_then(|v| v.checked_add(from.offset))
        .ok_or(ConvertError::Overflow)?;
    let out = base
        .checked_sub(dest.offset)
        .and_then(|v| v.checked_div(dest.factor))
        .ok_or(ConvertError::Overflow)?;
    Ok(Verdict::new(out.normalize()).because(Reason::new(
        "value.unit.convert",
        format!(
            "{} {} -> {} via base ({} {}): value*{}+{} then (b-{})/{}",
            q.value,
            q.unit,
            to,
            base.normalize(),
            from.dimension,
            from.factor,
            from.offset,
            dest.offset,
            dest.factor
        ),
    )))
}

/// Quantities: a number (digits or words) followed, after at most one space, by a unit surface from the table.
/// The longest surface wins. The found span covers number and unit.
pub fn parse_quantities(
    text: &str,
    numbers: &NumberRules,
    table: &UnitTable,
) -> Vec<Verdict<Found<Quantity>>> {
    let mut nums: Vec<(Span, Decimal, String)> = scan_digits(text, numbers)
        .into_iter()
        .map(|r| (r.span, r.value, r.note))
        .collect();
    nums.extend(scan_words(text, numbers));
    nums.sort_by_key(|n| n.0);
    let mut out = Vec::new();
    let mut floor = 0usize;
    for (span, value, _) in nums {
        if span.start() < floor {
            continue;
        }
        let rest = &text[span.end()..];
        let after = rest.strip_prefix([' ', '\u{a0}']).unwrap_or(rest);
        let skipped = rest.len() - after.len();
        if let Some((surface_len, id)) = longest_surface(after, table) {
            let end = span.end() + skipped + surface_len;
            let q_span = Span::new(span.start(), end).expect("ordered");
            let def = table.get(id).expect("surface ids are defined");
            out.push(
                Verdict::new(Found {
                    span: q_span,
                    value: Quantity {
                        value,
                        unit: id.to_string(),
                    },
                })
                .because(Reason::at(
                    "value.quantity",
                    q_span,
                    format!(
                        "{value} followed by unit {:?} ({})",
                        &text[span.end() + skipped..end],
                        def.dimension
                    ),
                )),
            );
            floor = end;
        }
    }
    out
}

/// The longest unit surface at the start of `s` (up to `max_words` space-separated words), ending at a boundary.
fn longest_surface<'a>(s: &str, table: &'a UnitTable) -> Option<(usize, &'a str)> {
    let mut ends = vec![];
    let mut pos = 0;
    for (n, w) in s.split(' ').enumerate() {
        if n >= table.max_words {
            break;
        }
        pos += w.len();
        ends.push(pos);
        pos += 1;
    }
    for &e in ends.iter().rev() {
        // trim trailing sentence punctuation from the candidate
        let mut cand = &s[..e];
        while let Some(c) = cand.chars().next_back().filter(|c| {
            matches!(
                c,
                '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '"' | '\''
            )
        }) {
            let trimmed = &cand[..cand.len() - c.len_utf8()];
            if table.surfaces.contains_key(cand) {
                break;
            }
            cand = trimmed;
        }
        if let Some(id) = table.surfaces.get(cand) {
            // candidates are whole space-separated words minus trailing punctuation, so a hit is always at a boundary
            return Some((cand.len(), id.as_str()));
        }
    }
    None
}
