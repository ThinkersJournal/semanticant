use crate::{Found, ValueError};
use semanticant_core::{Reason, Span, Verdict};
use semanticant_pack::Pack;
use std::collections::{HashMap, HashSet};

/// A calendar date that may be partial: `year`, `month`, `day` are each present or absent. Never padded or guessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PartialDate {
    /// Year, if the text gave one (four digits).
    pub year: Option<i32>,
    /// Month 1-12, if given.
    pub month: Option<u8>,
    /// Day of month, if given (a day needs a month).
    pub day: Option<u8>,
}

/// A time of day (no zone, no date).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeOfDay {
    /// 0-23.
    pub hour: u8,
    /// 0-59.
    pub minute: u8,
    /// 0-59.
    pub second: u8,
}

/// A date in the text with EVERY reading it supports (more than one when the order of day and month is ambiguous).
#[derive(Debug, Clone, PartialEq)]
pub struct DateMatch {
    /// Where it is.
    pub span: Span,
    /// One or more readings, each with its reasons. Day-first before month-first.
    pub readings: Vec<Verdict<PartialDate>>,
}

/// Month names, ordinal suffixes and am/pm words of a language. DATA from a pack or the caller.
#[derive(Debug, Clone, Default)]
pub struct CalendarRules {
    months: HashMap<String, u8>,
    ordinals: HashSet<String>,
    am: HashSet<String>,
    pm: HashSet<String>,
}

impl CalendarRules {
    /// Empty rules (only numeric dates and 24-hour times will match).
    pub fn new() -> Self {
        Self::default()
    }
    /// Add a month name or abbreviation (case-insensitive) for month `1..=12`.
    pub fn with_month(mut self, name: &str, month: u8) -> Self {
        assert!((1..=12).contains(&month), "month must be 1..=12");
        self.months.insert(name.to_lowercase(), month);
        self
    }
    /// Add an ordinal suffix ("st", "nd", "rd", "th").
    pub fn with_ordinal(mut self, suffix: &str) -> Self {
        self.ordinals.insert(suffix.to_lowercase());
        self
    }
    /// Add a before-noon marker ("am").
    pub fn with_am(mut self, w: &str) -> Self {
        self.am.insert(w.to_lowercase());
        self
    }
    /// Add an after-noon marker ("pm").
    pub fn with_pm(mut self, w: &str) -> Self {
        self.pm.insert(w.to_lowercase());
        self
    }
    /// Build from a pack of `kind = "calendar"`: `months.tsv` (`name<TAB>1..12`, required), `ordinals.tsv` (one suffix per row) and
    /// `meridiem.tsv` (`word<TAB>am|pm`), both optional.
    pub fn from_pack(pack: &Pack) -> Result<Self, ValueError> {
        let kind = &pack.manifest().kind;
        if kind != "calendar" {
            return Err(ValueError::WrongKind {
                found: kind.clone(),
                want: "calendar",
            });
        }
        let mut r = Self::new();
        for (n, row) in pack
            .tsv("months.tsv")
            .ok_or(ValueError::MissingFile("months.tsv"))?
            .iter()
            .enumerate()
        {
            let m: u8 = row
                .get(1)
                .and_then(|m| m.trim().parse().ok())
                .filter(|m| (1..=12).contains(m))
                .ok_or(ValueError::BadRow(
                    "months.tsv",
                    n + 1,
                    "want name<TAB>1..12",
                ))?;
            r = r.with_month(row[0], m);
        }
        for row in pack.tsv("ordinals.tsv").unwrap_or_default() {
            r = r.with_ordinal(row[0]);
        }
        for (n, row) in pack
            .tsv("meridiem.tsv")
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            match row.get(1).copied() {
                Some("am") => r = r.with_am(row[0]),
                Some("pm") => r = r.with_pm(row[0]),
                _ => {
                    return Err(ValueError::BadRow(
                        "meridiem.tsv",
                        n + 1,
                        "want word<TAB>am|pm",
                    ));
                }
            }
        }
        Ok(r)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Num,
    Alpha,
    Space,
    Punct(char),
}
#[derive(Clone, Copy)]
struct Tk {
    kind: Kind,
    start: usize,
    end: usize,
}

fn tokenise(text: &str) -> Vec<Tk> {
    let mut out: Vec<Tk> = Vec::new();
    let class = |c: char| -> Kind {
        if c.is_ascii_digit() {
            Kind::Num
        } else if c.is_alphabetic() {
            Kind::Alpha
        } else if matches!(c, ' ' | '\t' | '\u{a0}' | '\u{202f}') {
            Kind::Space
        } else {
            Kind::Punct(c)
        }
    };
    for (i, c) in text.char_indices() {
        let k = class(c);
        match (out.last_mut(), k) {
            (Some(t), k2)
                if t.kind == k2 && matches!(k2, Kind::Num | Kind::Alpha | Kind::Space) =>
            {
                t.end = i + c.len_utf8()
            }
            _ => out.push(Tk {
                kind: k,
                start: i,
                end: i + c.len_utf8(),
            }),
        }
    }
    out
}

fn num_len(text: &str, t: &Tk) -> Option<usize> {
    (t.kind == Kind::Num).then(|| text[t.start..t.end].len())
}
fn num(text: &str, t: &Tk) -> u32 {
    text[t.start..t.end].parse().unwrap_or(u32::MAX)
}
fn punct(t: Option<&Tk>) -> Option<char> {
    match t.map(|t| t.kind) {
        Some(Kind::Punct(c)) => Some(c),
        _ => None,
    }
}

/// Is this (year?, month, day?) a real calendar date? Uses `jiff` for the calendar. A day without a year is checked against a leap year.
fn valid(year: Option<i32>, month: u32, day: Option<u32>) -> bool {
    if !(1..=12).contains(&month) {
        return false;
    }
    match day {
        None => true,
        Some(d) => jiff::civil::Date::new(
            year.map_or(2000, |y| y as i16),
            month as i8,
            d.min(127) as i8,
        )
        .is_ok(),
    }
}

/// A recognised date shape that is not a real date: consume its tokens (empty readings) so no part of it matches on its own.
fn gate(
    ok: bool,
    f: impl FnOnce() -> (usize, Vec<Verdict<PartialDate>>),
) -> Option<(usize, Vec<Verdict<PartialDate>>)> {
    let (end, rs) = f();
    Some((end, if ok { rs } else { Vec::new() }))
}

fn reading(
    year: Option<i32>,
    month: u32,
    day: Option<u32>,
    span: Span,
    rule: &'static str,
    note: &str,
) -> Verdict<PartialDate> {
    Verdict::new(PartialDate {
        year,
        month: Some(month as u8),
        day: day.map(|d| d as u8),
    })
    .because(Reason::at(rule, span, note))
}

/// Dates in `text`, in order, non-overlapping. Recognised shapes: `YYYY-MM-DD` (also `/` or `.`) and `YYYY-MM`; `A/B/YYYY` (also `-`, `.`),
/// which yields a day-first and a month-first reading when both are real dates; `12 March 2026`, `12th March`, `March 12, 2026`, `March 2026`
/// with month names from `rules`. Impossible dates (30 Feb) are not returned. Only four-digit years are read. A month name that is also an
/// ordinary word in the language (English "may") will match in running text: that is the pack's trade-off.
pub fn parse_dates(text: &str, rules: &CalendarRules) -> Vec<DateMatch> {
    let toks = tokenise(text);
    let mut out = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        match date_at(text, &toks, i, rules) {
            Some((next, readings)) if readings.is_empty() => i = next,
            Some((next, readings)) => {
                let span = Span::new(toks[i].start, toks[next - 1].end).expect("ordered");
                let readings = readings
                    .into_iter()
                    .map(|v| Verdict {
                        value: v.value,
                        reasons: v
                            .reasons
                            .into_iter()
                            .map(|r| Reason {
                                span: Some(span),
                                ..r
                            })
                            .collect(),
                    })
                    .collect();
                out.push(DateMatch { span, readings });
                i = next;
            }
            None => i += 1,
        }
    }
    out
}

fn date_at(
    text: &str,
    t: &[Tk],
    i: usize,
    rules: &CalendarRules,
) -> Option<(usize, Vec<Verdict<PartialDate>>)> {
    let sp = Span::new(t[i].start, t[i].end).expect("ordered");
    let none = || -> Option<(usize, Vec<Verdict<PartialDate>>)> { None };
    match t[i].kind {
        Kind::Num => {
            let l0 = num_len(text, &t[i])?;
            let sep = punct(t.get(i + 1));
            if l0 == 4
                && matches!(sep, Some('-' | '/' | '.'))
                && t.get(i + 2)
                    .and_then(|x| num_len(text, x))
                    .is_some_and(|l| (1..=2).contains(&l))
            {
                let (y, m) = (num(text, &t[i]) as i32, num(text, &t[i + 2]));
                if punct(t.get(i + 3)) == sep
                    && t.get(i + 4)
                        .and_then(|x| num_len(text, x))
                        .is_some_and(|l| (1..=2).contains(&l))
                {
                    let d = num(text, &t[i + 4]);
                    return gate(valid(Some(y), m, Some(d)), || {
                        (
                            i + 5,
                            vec![reading(
                                Some(y),
                                m,
                                Some(d),
                                sp,
                                "value.date.iso",
                                "year first: year, month, day",
                            )],
                        )
                    });
                }
                return gate(valid(Some(y), m, None), || {
                    (
                        i + 3,
                        vec![reading(
                            Some(y),
                            m,
                            None,
                            sp,
                            "value.date.iso",
                            "year first: year, month",
                        )],
                    )
                });
            }
            if (1..=2).contains(&l0) {
                if matches!(sep, Some('/' | '-' | '.'))
                    && t.get(i + 2)
                        .and_then(|x| num_len(text, x))
                        .is_some_and(|l| (1..=2).contains(&l))
                    && punct(t.get(i + 3)) == sep
                    && t.get(i + 4).and_then(|x| num_len(text, x)) == Some(4)
                {
                    let (a, b, y) = (
                        num(text, &t[i]),
                        num(text, &t[i + 2]),
                        num(text, &t[i + 4]) as i32,
                    );
                    let mut rs = Vec::new();
                    if valid(Some(y), b, Some(a)) {
                        rs.push(reading(
                            Some(y),
                            b,
                            Some(a),
                            sp,
                            "value.date.numeric.dmy",
                            "read day-month-year",
                        ));
                    }
                    if a != b && valid(Some(y), a, Some(b)) {
                        rs.push(reading(
                            Some(y),
                            a,
                            Some(b),
                            sp,
                            "value.date.numeric.mdy",
                            "read month-day-year",
                        ));
                    }
                    if rs.len() > 1 {
                        for r in &mut rs {
                            r.reasons.push(Reason::at(
                                "value.date.ambiguous",
                                sp,
                                "both day-first and month-first are real dates",
                            ));
                        }
                    }
                    return Some((i + 5, rs));
                }
                // day, optional ordinal, month name, optional year
                let day = num(text, &t[i]);
                let mut j = i + 1;
                if t.get(j).is_some_and(|x| {
                    x.kind == Kind::Alpha
                        && rules
                            .ordinals
                            .contains(&text[x.start..x.end].to_lowercase())
                }) {
                    j += 1;
                }
                if t.get(j).is_some_and(|x| x.kind == Kind::Space)
                    && t.get(j + 1).is_some_and(|x| x.kind == Kind::Alpha)
                    && let Some(&month) = rules
                        .months
                        .get(&text[t[j + 1].start..t[j + 1].end].to_lowercase())
                {
                    {
                        let mut end = j + 2;
                        let mut year = None;
                        let (mut k, mut ok) = (end, true);
                        if punct(t.get(k)) == Some(',') {
                            k += 1;
                        }
                        if t.get(k).is_some_and(|x| x.kind == Kind::Space) {
                            k += 1;
                        } else {
                            ok = false;
                        }
                        if ok && t.get(k).and_then(|x| num_len(text, x)) == Some(4) {
                            year = Some(num(text, &t[k]) as i32);
                            end = k + 1;
                        }
                        return gate(valid(year, u32::from(month), Some(day)), || {
                            (
                                end,
                                vec![reading(
                                    year,
                                    u32::from(month),
                                    Some(day),
                                    sp,
                                    "value.date.textual",
                                    "day, month name, optional year",
                                )],
                            )
                        });
                    }
                }
            }
            none()
        }
        Kind::Alpha => {
            let &month = rules
                .months
                .get(&text[t[i].start..t[i].end].to_lowercase())?;
            let month = u32::from(month);
            if !t.get(i + 1).is_some_and(|x| x.kind == Kind::Space) {
                return none();
            }
            let n1 = t.get(i + 2)?;
            let l = num_len(text, n1)?;
            if l == 4 {
                let y = num(text, n1) as i32;
                return Some((
                    i + 3,
                    vec![reading(
                        Some(y),
                        month,
                        None,
                        sp,
                        "value.date.textual",
                        "month name and year",
                    )],
                ));
            }
            if !(1..=2).contains(&l) {
                return none();
            }
            let day = num(text, n1);
            let mut j = i + 3;
            if t.get(j).is_some_and(|x| {
                x.kind == Kind::Alpha
                    && rules
                        .ordinals
                        .contains(&text[x.start..x.end].to_lowercase())
            }) {
                j += 1;
            }
            let (mut end, mut year) = (j, None);
            let mut k = j;
            if punct(t.get(k)) == Some(',') {
                k += 1;
            }
            if t.get(k).is_some_and(|x| x.kind == Kind::Space)
                && t.get(k + 1).and_then(|x| num_len(text, x)) == Some(4)
            {
                year = Some(num(text, &t[k + 1]) as i32);
                end = k + 2;
            }
            gate(valid(year, month, Some(day)), || {
                (
                    end,
                    vec![reading(
                        year,
                        month,
                        Some(day),
                        sp,
                        "value.date.textual",
                        "month name, day, optional year",
                    )],
                )
            })
        }
        _ => none(),
    }
}

/// Times of day: `H:MM`, `H:MM:SS` (24-hour unless followed by a marker) and `H am` / `H:MM pm` with markers from `rules`.
/// A 12-hour time needs hour 1-12; `12 am` is 00:00 and `12 pm` is 12:00. Out-of-range times are not returned. Zones are out of scope.
pub fn parse_times(text: &str, rules: &CalendarRules) -> Vec<Verdict<Found<TimeOfDay>>> {
    let t = tokenise(text);
    let mut out = Vec::new();
    let mut i = 0;
    while i < t.len() {
        let Some(l0) = num_len(text, &t[i]).filter(|l| (1..=2).contains(l)) else {
            i += 1;
            continue;
        };
        let _ = l0;
        let hour = num(text, &t[i]);
        let (mut minute, mut second, mut j) = (0u32, 0u32, i + 1);
        let mut has_min = false;
        if punct(t.get(j)) == Some(':') && t.get(j + 1).and_then(|x| num_len(text, x)) == Some(2) {
            minute = num(text, &t[j + 1]);
            j += 2;
            has_min = true;
            if punct(t.get(j)) == Some(':')
                && t.get(j + 1).and_then(|x| num_len(text, x)) == Some(2)
            {
                second = num(text, &t[j + 1]);
                j += 2;
            }
        }
        // optional meridiem after at most one space
        let mut m_idx = j;
        if t.get(m_idx).is_some_and(|x| x.kind == Kind::Space) && t[m_idx].end - t[m_idx].start <= 3
        {
            m_idx += 1;
        }
        let marker = t
            .get(m_idx)
            .filter(|x| x.kind == Kind::Alpha)
            .map(|x| text[x.start..x.end].to_lowercase());
        let (is_am, is_pm) = marker.as_ref().map_or((false, false), |m| {
            (rules.am.contains(m), rules.pm.contains(m))
        });
        let (h24, end, rule, note) = if is_am || is_pm {
            if !(1..=12).contains(&hour) {
                i += 1;
                continue;
            }
            let h = if is_am { hour % 12 } else { hour % 12 + 12 };
            (h, m_idx + 1, "value.time.12h", "12-hour clock with marker")
        } else if has_min {
            (hour, j, "value.time.24h", "24-hour clock")
        } else {
            i += 1;
            continue;
        };
        if jiff::civil::Time::new(h24 as i8, minute as i8, second as i8, 0).is_err() {
            i += 1;
            continue;
        }
        let span = Span::new(t[i].start, t[end - 1].end).expect("ordered");
        out.push(
            Verdict::new(Found {
                span,
                value: TimeOfDay {
                    hour: h24 as u8,
                    minute: minute as u8,
                    second: second as u8,
                },
            })
            .because(Reason::at(rule, span, note)),
        );
        i = end;
    }
    out
}
