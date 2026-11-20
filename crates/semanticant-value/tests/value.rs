#![allow(missing_docs)]
use semanticant_core::pack::{FileEntry, Manifest, SCHEMA_VERSION, Sha256Hex};
use semanticant_core::{LangTag, Provenance};
use semanticant_pack::{Pack, sha256_hex};
use semanticant_value::*;
use std::collections::HashMap;
use std::str::FromStr;

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}
fn pack(kind: &str, files: &[(&str, &str)]) -> Pack {
    let m = Manifest {
        name: "t".into(),
        version: "1".into(),
        kind: kind.into(),
        schema: SCHEMA_VERSION,
        lang: Some(LangTag::parse("en").unwrap()),
        provenance: Provenance {
            source: "test".into(),
            licence: "MIT".into(),
            credit: "t".into(),
            url: None,
            retrieved: None,
        },
        files: files
            .iter()
            .map(|(n, b)| FileEntry {
                name: (*n).into(),
                sha256: Sha256Hex::parse(&sha256_hex(b.as_bytes())).unwrap(),
            })
            .collect(),
    };
    let data: HashMap<String, Vec<u8>> = files
        .iter()
        .map(|(n, b)| ((*n).to_string(), b.as_bytes().to_vec()))
        .collect();
    Pack::from_parts(m, data).unwrap()
}

const WORDS: &str = "zero\tunit\t0\none\tunit\t1\ntwo\tunit\t2\nthree\tunit\t3\nfour\tunit\t4\nfive\tunit\t5\nsix\tunit\t6\nseven\tunit\t7\neight\tunit\t8\nnine\tunit\t9
ten\tunit\t10\nseventeen\tunit\t17\ntwenty\ttens\t20\nthirty\ttens\t30\nforty\ttens\t40\nhundred\tscale\t100\nthousand\tscale\t1000\nmillion\tscale\t1000000\nand\tconnector\n";

fn en() -> NumberRules {
    NumberRules::from_pack(&pack("number-words", &[("words.tsv", WORDS)]), '.', &[',']).unwrap()
}
fn nums(text: &str, r: &NumberRules) -> Vec<(String, Decimal)> {
    parse_numbers(text, r)
        .into_iter()
        .map(|v| {
            (
                text[v.value.span.start()..v.value.span.end()].to_string(),
                v.value.value,
            )
        })
        .collect()
}

#[test]
fn digit_numbers() {
    let r = en();
    assert_eq!(
        nums("I have 1,234 apples and 5.5 pears", &r),
        vec![("1,234".into(), d("1234")), ("5.5".into(), d("5.5"))]
    );
    assert_eq!(
        nums("1,234,567.89", &r),
        vec![("1,234,567.89".into(), d("1234567.89"))]
    );
    assert_eq!(
        nums("1,23", &r),
        vec![("1".into(), d("1")), ("23".into(), d("23"))],
        "not a group of three"
    );
    assert_eq!(
        nums("1234,567", &r),
        vec![("1234".into(), d("1234")), ("567".into(), d("567"))],
        "first run too long to be grouped"
    );
    assert_eq!(
        nums("1,2345", &r),
        vec![("1".into(), d("1")), ("2345".into(), d("2345"))]
    );
    assert_eq!(
        nums("-5 and +7 and \u{2212}9", &r),
        vec![
            ("-5".into(), d("-5")),
            ("+7".into(), d("7")),
            ("\u{2212}9".into(), d("-9"))
        ]
    );
    assert_eq!(
        nums("pages 3-5", &r),
        vec![("3".into(), d("3")), ("5".into(), d("5"))],
        "a hyphen between numbers is not a sign"
    );
    assert_eq!(nums("x-5", &r), vec![("5".into(), d("5"))]);
    assert_eq!(
        nums("H2O 3rd 5km", &r),
        vec![],
        "letters attached on either side: not a plain number"
    );
    assert_eq!(nums("0.50", &r), vec![("0.50".into(), d("0.50"))]);
    assert_eq!(
        nums("99999999999999999999999999999999 7", &r),
        vec![("7".into(), d("7"))],
        "beyond Decimal is skipped"
    );
    let eu = NumberRules::new(',', &['.']);
    assert_eq!(nums("1.234,5", &eu), vec![("1.234,5".into(), d("1234.5"))]);
    assert_eq!(nums("1,5", &eu), vec![("1,5".into(), d("1.5"))]);
}

#[test]
fn spelled_out_numbers() {
    let r = en();
    let one = |t: &str| nums(t, &r);
    assert_eq!(one("twenty-one"), vec![("twenty-one".into(), d("21"))]);
    assert_eq!(one("Twenty-One"), vec![("Twenty-One".into(), d("21"))]);
    assert_eq!(
        one("one hundred and five"),
        vec![("one hundred and five".into(), d("105"))]
    );
    assert_eq!(
        one("two thousand three hundred forty five"),
        vec![("two thousand three hundred forty five".into(), d("2345"))]
    );
    assert_eq!(
        one("two thousand three hundred forty-two"),
        vec![("two thousand three hundred forty-two".into(), d("2342"))]
    );
    assert_eq!(
        one("forty-two thousand"),
        vec![("forty-two thousand".into(), d("42000"))]
    );
    assert_eq!(
        one("one million"),
        vec![("one million".into(), d("1000000"))]
    );
    assert_eq!(one("hundred"), vec![("hundred".into(), d("100"))]);
    assert_eq!(one("zero"), vec![("zero".into(), d("0"))]);
    // sequences that are two numbers, not one
    assert_eq!(
        one("one two"),
        vec![("one".into(), d("1")), ("two".into(), d("2"))]
    );
    assert_eq!(
        one("twenty twenty"),
        vec![("twenty".into(), d("20")), ("twenty".into(), d("20"))]
    );
    assert_eq!(
        one("ten five"),
        vec![("ten".into(), d("10")), ("five".into(), d("5"))]
    );
    assert_eq!(
        one("twenty ten"),
        vec![("twenty".into(), d("20")), ("ten".into(), d("10"))]
    );
    assert_eq!(
        one("one thousand one million"),
        vec![
            ("one thousand one".into(), d("1001")),
            ("million".into(), d("1000000"))
        ],
        "a larger scale cannot follow a smaller one: it starts its own number"
    );
    assert_eq!(
        one("one thousand million"),
        vec![
            ("one thousand".into(), d("1000")),
            ("million".into(), d("1000000"))
        ]
    );
    assert_eq!(
        one("one hundred hundred"),
        vec![
            ("one hundred".into(), d("100")),
            ("hundred".into(), d("100"))
        ]
    );
    // a connector only counts between number words
    assert_eq!(
        one("one hundred and"),
        vec![("one hundred".into(), d("100"))]
    );
    assert_eq!(one("salt and one"), vec![("one".into(), d("1"))]);
    assert_eq!(
        one("one, two"),
        vec![("one".into(), d("1")), ("two".into(), d("2"))]
    );
    // mixed with digits, in text order
    assert_eq!(
        one("5 apples and seven pears"),
        vec![("5".into(), d("5")), ("seven".into(), d("7"))]
    );
    let v = &parse_numbers("seven", &r)[0];
    assert_eq!(v.reasons[0].rule, "value.number.words");
    assert_eq!(
        parse_numbers("7", &r)[0].reasons[0].rule,
        "value.number.digits"
    );
}

#[test]
fn number_word_packs_are_validated() {
    assert!(matches!(
        NumberRules::from_pack(&pack("units", &[("words.tsv", WORDS)]), '.', &[]),
        Err(ValueError::WrongKind { .. })
    ));
    assert!(matches!(
        NumberRules::from_pack(&pack("number-words", &[("x.tsv", "a")]), '.', &[]),
        Err(ValueError::MissingFile("words.tsv"))
    ));
    for bad in [
        "one\tunit\t25\n",
        "twenty\ttens\t5\n",
        "ten\tscale\t10\n",
        "one\tunit\tx\n",
        "one\n",
        "one\tmagic\t1\n",
    ] {
        assert!(
            matches!(
                NumberRules::from_pack(&pack("number-words", &[("words.tsv", bad)]), '.', &[]),
                Err(ValueError::BadRow("words.tsv", 1, _))
            ),
            "{bad:?}"
        );
    }
}

// base units: length m, mass kg, temperature degrees Fahrenheit (so C and K have exact decimal factors).
const UNITS: &str = "m\tm\tlength\t1\nmetres\tm\tlength\t1\nkm\tkm\tlength\t1000\nkilometres\tkm\tlength\t1000\ncm\tcm\tlength\t0.01
mile\tmile\tlength\t1609.344\nsquare metres\tm2\tarea\t1\nkg\tkg\tmass\t1\nlb\tlb\tmass\t0.45359237
°F\tF\ttemperature\t1\n°C\tC\ttemperature\t1.8\t32\nK\tK\ttemperature\t1.8\t-459.67\n%\tpercent\tratio\t0.01\n";

fn table() -> UnitTable {
    UnitTable::from_pack(&pack("units", &[("units.tsv", UNITS)])).unwrap()
}
fn quants(text: &str) -> Vec<(String, Decimal, String)> {
    parse_quantities(text, &en(), &table())
        .into_iter()
        .map(|v| {
            (
                text[v.value.span.start()..v.value.span.end()].to_string(),
                v.value.value.value,
                v.value.value.unit,
            )
        })
        .collect()
}

#[test]
fn quantities_in_text() {
    assert_eq!(
        quants("The road is 5 km long"),
        vec![("5 km".into(), d("5"), "km".into())]
    );
    assert_eq!(quants("5km"), vec![("5km".into(), d("5"), "km".into())]);
    assert_eq!(
        quants("It is 12.5 kilometres."),
        vec![("12.5 kilometres".into(), d("12.5"), "km".into())]
    );
    assert_eq!(
        quants("five kilometres away"),
        vec![("five kilometres".into(), d("5"), "km".into())]
    );
    assert_eq!(
        quants("a 3 square metres room"),
        vec![("3 square metres".into(), d("3"), "m2".into())],
        "multi-word surface"
    );
    assert_eq!(
        quants("100 \u{b0}C and 50%"),
        vec![
            ("100 \u{b0}C".into(), d("100"), "C".into()),
            ("50%".into(), d("50"), "percent".into())
        ]
    );
    assert_eq!(quants("Run 5 m."), vec![("5 m".into(), d("5"), "m".into())]);
    assert_eq!(
        quants("(10 kg)"),
        vec![("10 kg".into(), d("10"), "kg".into())]
    );
    assert_eq!(
        quants("wait 5 min"),
        vec![],
        "m is a prefix of min, not a unit here"
    );
    assert_eq!(quants("5 kmh"), vec![]);
    assert_eq!(quants("just 5 apples"), vec![]);
    assert_eq!(
        quants("5\u{a0}km"),
        vec![("5\u{a0}km".into(), d("5"), "km".into())]
    );
    assert_eq!(
        quants("3 km, then 4 miles"),
        vec![("3 km".into(), d("3"), "km".into())],
        "miles is not the surface 'mile'"
    );
    let v = &parse_quantities("5 km", &en(), &table())[0];
    assert_eq!(v.reasons[0].rule, "value.quantity");
    assert_eq!(v.value.span.slice("5 km"), Some("5 km"));
}

fn q(v: &str, u: &str) -> Quantity {
    Quantity {
        value: d(v),
        unit: u.into(),
    }
}
fn conv(v: &str, from: &str, to: &str) -> Result<Decimal, ConvertError> {
    convert(&q(v, from), to, &table()).map(|v| v.value)
}

#[test]
fn conversions_are_exact() {
    assert_eq!(conv("1.5", "km", "m"), Ok(d("1500")));
    assert_eq!(conv("2", "mile", "m"), Ok(d("3218.688")));
    assert_eq!(conv("10", "lb", "kg"), Ok(d("4.5359237")));
    assert_eq!(conv("100", "C", "F"), Ok(d("212")));
    assert_eq!(conv("212", "F", "C"), Ok(d("100")));
    assert_eq!(conv("32", "F", "C"), Ok(d("0")));
    assert_eq!(conv("0", "C", "K"), Ok(d("273.15")));
    assert_eq!(conv("273.15", "K", "C"), Ok(d("0")));
    assert_eq!(conv("5", "km", "km"), Ok(d("5")));
    assert_eq!(conv("1", "cm", "km"), Ok(d("0.00001")));
    assert_eq!(
        conv("5", "km", "kg"),
        Err(ConvertError::DimensionMismatch(
            "length".into(),
            "mass".into()
        ))
    );
    assert_eq!(
        conv("5", "furlong", "m"),
        Err(ConvertError::UnknownUnit("furlong".into()))
    );
    assert_eq!(
        conv("5", "m", "furlong"),
        Err(ConvertError::UnknownUnit("furlong".into()))
    );
    assert_eq!(
        convert(
            &Quantity {
                value: Decimal::MAX,
                unit: "km".into()
            },
            "m",
            &table()
        )
        .unwrap_err(),
        ConvertError::Overflow
    );
    let v = convert(&q("1", "km"), "m", &table()).unwrap();
    assert_eq!(v.reasons[0].rule, "value.unit.convert");
}

#[test]
fn unit_packs_are_validated() {
    let bad = |rows: &str| UnitTable::from_pack(&pack("units", &[("units.tsv", rows)]));
    assert!(matches!(
        bad("m\tm\tlength\t0\n"),
        Err(ValueError::BadRow("units.tsv", 1, _))
    ));
    assert!(matches!(
        bad("m\tm\tlength\tx\n"),
        Err(ValueError::BadRow(..))
    ));
    assert!(matches!(bad("m\tm\tlength\n"), Err(ValueError::BadRow(..))));
    assert!(matches!(
        bad("m\tm\tlength\t1\t?\n"),
        Err(ValueError::BadRow(..))
    ));
    assert_eq!(
        bad("m\tm\tlength\t1\nmetre\tm\tlength\t2\n").unwrap_err(),
        ValueError::ConflictingUnit("m".into())
    );
    assert!(matches!(
        UnitTable::from_pack(&pack("number-words", &[("units.tsv", UNITS)])),
        Err(ValueError::WrongKind { .. })
    ));
    let t = table();
    assert_eq!(t.get("km").unwrap().factor, d("1000"));
    assert!(t.get("nope").is_none());
}

const MONTHS: &str = "january\t1\nfebruary\t2\nmarch\t3\napril\t4\nmay\t5\njune\t6\njuly\t7\naugust\t8\nseptember\t9\noctober\t10\nnovember\t11\ndecember\t12\nmar\t3\nsept\t9\n";

fn cal() -> CalendarRules {
    CalendarRules::from_pack(&pack(
        "calendar",
        &[
            ("months.tsv", MONTHS),
            ("ordinals.tsv", "st\nnd\nrd\nth\n"),
            ("meridiem.tsv", "am\tam\npm\tpm\n"),
        ],
    ))
    .unwrap()
}
fn pd(y: Option<i32>, m: Option<u8>, d: Option<u8>) -> PartialDate {
    PartialDate {
        year: y,
        month: m,
        day: d,
    }
}
/// (matched text, readings)
fn dates(text: &str) -> Vec<(String, Vec<PartialDate>)> {
    parse_dates(text, &cal())
        .into_iter()
        .map(|m| {
            (
                text[m.span.start()..m.span.end()].to_string(),
                m.readings.into_iter().map(|r| r.value).collect(),
            )
        })
        .collect()
}
fn one(text: &str) -> Vec<PartialDate> {
    let mut v = dates(text);
    assert_eq!(v.len(), 1, "{text}: {v:?}");
    v.remove(0).1
}

#[test]
fn iso_dates_and_calendar_validity() {
    assert_eq!(one("2026-11-18"), vec![pd(Some(2026), Some(11), Some(18))]);
    assert_eq!(one("2026/11/18"), vec![pd(Some(2026), Some(11), Some(18))]);
    assert_eq!(one("2026.11.18"), vec![pd(Some(2026), Some(11), Some(18))]);
    assert_eq!(
        one("2026-11"),
        vec![pd(Some(2026), Some(11), None)],
        "partial stays partial"
    );
    assert_eq!(one("2024-02-29"), vec![pd(Some(2024), Some(2), Some(29))]);
    for bad in [
        "2026-02-29",
        "2026-02-30",
        "2026-13-01",
        "2026-00-10",
        "2026-04-31",
        "2026-13",
    ] {
        assert!(dates(bad).is_empty(), "{bad}");
    }
    assert!(dates("2026-11-18-5").len() <= 1);
    assert_eq!(
        dates("from 2026-01-02 to 2026-03-04"),
        vec![
            ("2026-01-02".into(), vec![pd(Some(2026), Some(1), Some(2))]),
            ("2026-03-04".into(), vec![pd(Some(2026), Some(3), Some(4))])
        ]
    );
    let m = &parse_dates("2026-11-18", &cal())[0];
    assert_eq!(m.readings[0].reasons[0].rule, "value.date.iso");
}

#[test]
fn ambiguous_numeric_dates_return_every_reading() {
    let r = one("03/04/2026");
    assert_eq!(
        r,
        vec![
            pd(Some(2026), Some(4), Some(3)),
            pd(Some(2026), Some(3), Some(4))
        ],
        "day-first then month-first"
    );
    let m = &parse_dates("03/04/2026", &cal())[0];
    assert_eq!(m.readings[0].reasons[0].rule, "value.date.numeric.dmy");
    assert_eq!(m.readings[1].reasons[0].rule, "value.date.numeric.mdy");
    assert!(
        m.readings
            .iter()
            .all(|r| r.reasons.iter().any(|x| x.rule == "value.date.ambiguous"))
    );
    assert_eq!(one("13/04/2026"), vec![pd(Some(2026), Some(4), Some(13))]);
    assert_eq!(one("04/13/2026"), vec![pd(Some(2026), Some(4), Some(13))]);
    let same = parse_dates("04/04/2026", &cal());
    assert_eq!(same[0].readings.len(), 1);
    assert!(
        same[0].readings[0]
            .reasons
            .iter()
            .all(|r| r.rule != "value.date.ambiguous"),
        "one reading is not ambiguous"
    );
    assert!(dates("31/04/2026").is_empty());
    assert_eq!(one("3.4.2026").len(), 2);
    assert_eq!(one("3-4-2026").len(), 2);
    assert!(dates("3/4/26").is_empty(), "two-digit years are not read");
    assert!(dates("3/4-2026").is_empty(), "separators must agree");
    assert_eq!(one("29/02/2024").len(), 1);
    assert!(dates("29/02/2026").is_empty());
}

#[test]
fn textual_dates() {
    assert_eq!(
        dates("Due 12 March 2026."),
        vec![(
            "12 March 2026".into(),
            vec![pd(Some(2026), Some(3), Some(12))]
        )]
    );
    assert_eq!(one("12th March"), vec![pd(None, Some(3), Some(12))]);
    assert_eq!(
        one("March 12, 2026"),
        vec![pd(Some(2026), Some(3), Some(12))]
    );
    assert_eq!(
        one("March 12 2026"),
        vec![pd(Some(2026), Some(3), Some(12))]
    );
    assert_eq!(one("March 2026"), vec![pd(Some(2026), Some(3), None)]);
    assert!(dates("1st of").is_empty());
    assert_eq!(one("MARCH 3rd"), vec![pd(None, Some(3), Some(3))]);
    assert_eq!(one("Sept 5"), vec![pd(None, Some(9), Some(5))]);
    assert_eq!(
        one("29 February"),
        vec![pd(None, Some(2), Some(29))],
        "a leap day with no year is allowed"
    );
    assert!(dates("30 February").is_empty());
    assert!(dates("31 February 2026").is_empty());
    assert!(dates("February 30, 2026").is_empty());
    assert!(dates("12 Marchx 2026").is_empty());
    assert!(dates("12 Unknownmonth 2026").is_empty());
    assert!(dates("March").is_empty());
    assert!(dates("March 123").is_empty());
    assert_eq!(
        dates("12 March 2026 and 1 April")[1].1,
        vec![pd(None, Some(4), Some(1))]
    );
    // the date after "12 March" is not swallowed as a year when it is not 4 digits
    assert_eq!(dates("12 March 14:30")[0].0, "12 March");
    assert_eq!(
        parse_dates("12 March 2026", &cal())[0].readings[0].reasons[0].rule,
        "value.date.textual"
    );
    // without month names in the rules, only numeric shapes match
    assert!(parse_dates("12 March 2026", &CalendarRules::new()).is_empty());
}

fn times(text: &str) -> Vec<(String, TimeOfDay)> {
    parse_times(text, &cal())
        .into_iter()
        .map(|v| {
            (
                text[v.value.span.start()..v.value.span.end()].to_string(),
                v.value.value,
            )
        })
        .collect()
}
fn tod(h: u8, m: u8, s: u8) -> TimeOfDay {
    TimeOfDay {
        hour: h,
        minute: m,
        second: s,
    }
}

#[test]
fn times_of_day() {
    assert_eq!(
        times("at 14:30 sharp"),
        vec![("14:30".into(), tod(14, 30, 0))]
    );
    assert_eq!(times("09:05:30"), vec![("09:05:30".into(), tod(9, 5, 30))]);
    assert_eq!(times("2:30 pm"), vec![("2:30 pm".into(), tod(14, 30, 0))]);
    assert_eq!(times("2:30pm"), vec![("2:30pm".into(), tod(14, 30, 0))]);
    assert_eq!(times("7PM"), vec![("7PM".into(), tod(19, 0, 0))]);
    assert_eq!(times("12 am"), vec![("12 am".into(), tod(0, 0, 0))]);
    assert_eq!(times("12 pm"), vec![("12 pm".into(), tod(12, 0, 0))]);
    assert_eq!(times("12:15 AM"), vec![("12:15 AM".into(), tod(0, 15, 0))]);
    assert_eq!(times("11:59 pm"), vec![("11:59 pm".into(), tod(23, 59, 0))]);
    assert_eq!(
        times("10:30 amazing"),
        vec![("10:30".into(), tod(10, 30, 0))],
        "amazing is not a marker"
    );
    for bad in [
        "25:00",
        "24:00",
        "10:61",
        "13 pm",
        "0 am",
        "9",
        "5 apples",
        "3 o'clock",
        "2026",
    ] {
        assert!(times(bad).is_empty(), "{bad}: {:?}", times(bad));
    }
    assert_eq!(
        times("2026-11-18T14:30"),
        vec![("14:30".into(), tod(14, 30, 0))]
    );
    assert_eq!(dates("2026-11-18T14:30")[0].0, "2026-11-18");
    assert_eq!(times("9:00 and 17:45")[1].1, tod(17, 45, 0));
    assert_eq!(
        parse_times("2:30 pm", &cal())[0].reasons[0].rule,
        "value.time.12h"
    );
    assert_eq!(
        parse_times("14:30", &cal())[0].reasons[0].rule,
        "value.time.24h"
    );
    // no markers in the rules: 12-hour forms do not match
    assert!(
        parse_times("2:30 pm", &CalendarRules::new())
            .iter()
            .all(|v| v.value.value.hour == 2)
    );
}

#[test]
fn calendar_packs_are_validated() {
    let bad = |m: &str, extra: &[(&str, &str)]| {
        let mut files = vec![("months.tsv", m)];
        files.extend_from_slice(extra);
        CalendarRules::from_pack(&pack("calendar", &files))
    };
    assert!(matches!(
        bad("march\t13\n", &[]),
        Err(ValueError::BadRow("months.tsv", 1, _))
    ));
    assert!(matches!(bad("march\n", &[]), Err(ValueError::BadRow(..))));
    assert!(matches!(
        bad("march\t3\n", &[("meridiem.tsv", "am\tmorning\n")]),
        Err(ValueError::BadRow("meridiem.tsv", 1, _))
    ));
    assert!(bad("march\t3\n", &[]).is_ok());
    assert!(matches!(
        CalendarRules::from_pack(&pack("units", &[("months.tsv", MONTHS)])),
        Err(ValueError::WrongKind { .. })
    ));
    assert!(matches!(
        CalendarRules::from_pack(&pack("calendar", &[("x.tsv", "a")])),
        Err(ValueError::MissingFile("months.tsv"))
    ));
}

#[test]
fn an_impossible_date_leaks_no_partial_match() {
    // each is a recognised full shape that is not a real date: nothing inside it may match on its own
    assert!(dates("31 February 2026").is_empty());
    assert!(dates("February 31, 2026").is_empty());
    assert!(dates("2026-02-30 2026").is_empty());
    assert!(dates("31/02/2026").is_empty());
    assert_eq!(dates("31 February 2026 then 1 March 2026").len(), 1);
}

#[test]
fn digits_glued_to_a_letter_before_them_are_not_numbers() {
    let r = en();
    assert_eq!(nums("x2 abc9 B12", &r), vec![]);
    assert_eq!(quants("abc5 km"), vec![]);
    assert_eq!(quants("x5km y"), vec![]);
}
