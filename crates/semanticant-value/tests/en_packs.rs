#![allow(missing_docs)]
//! The shipped English packs under `packs/en/` load, verify their checksums and drive the parsers.
use semanticant_pack::Pack;
use semanticant_value::*;
use std::path::PathBuf;

fn open(name: &str) -> Pack {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packs/en")
        .join(name);
    Pack::open(&dir).unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn english_packs_drive_the_parsers() {
    let nr = NumberRules::from_pack(&open("number-words"), '.', &[',']).unwrap();
    let t = "one hundred and twenty-three apples";
    let n = parse_numbers(t, &nr);
    assert_eq!(n.len(), 1);
    assert_eq!(n[0].value.value.to_string(), "123");

    let ut = UnitTable::from_pack(&open("units")).unwrap();
    let q = parse_quantities("walk 3 miles", &nr, &ut);
    assert_eq!(q.len(), 1);
    assert_eq!(q[0].value.value.unit, "mi");
    let c = parse_quantities("100 °C", &nr, &ut);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].value.value.unit, "C");

    let cal = CalendarRules::from_pack(&open("calendar")).unwrap();
    let d = parse_dates("on the 3rd of March 2026", &cal);
    assert!(!d.is_empty(), "textual date with an ordinal");
    let tm = parse_times("at 5 pm", &cal);
    assert_eq!(tm.len(), 1);
}

#[test]
fn tampered_pack_is_refused() {
    // Positive control for the checksum path: copy a pack, change one byte, expect refusal.
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packs/en/calendar");
    let dst = std::env::temp_dir().join(format!("sa-tamper-{}", std::process::id()));
    std::fs::create_dir_all(&dst).unwrap();
    for f in ["pack.toml", "months.tsv", "ordinals.tsv", "meridiem.tsv"] {
        std::fs::copy(src.join(f), dst.join(f)).unwrap();
    }
    assert!(Pack::open(&dst).is_ok(), "untouched copy opens");
    let mut m = std::fs::read(dst.join("months.tsv")).unwrap();
    m[0] ^= 1;
    std::fs::write(dst.join("months.tsv"), m).unwrap();
    assert!(Pack::open(&dst).is_err(), "one flipped byte is refused");
    std::fs::remove_dir_all(&dst).unwrap();
}

#[test]
fn german_calendar_pack_reads_german_dates() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packs/de/calendar");
    let cal = CalendarRules::from_pack(&Pack::open(&dir).unwrap()).unwrap();
    let d = parse_dates("am 3. März 2026", &cal);
    assert!(!d.is_empty(), "German month name and day");
    assert!(
        parse_dates("on 3 March 2026", &cal).is_empty(),
        "English month is not in the German pack"
    );
}

#[test]
fn french_calendar_pack_reads_french_dates() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packs/fr/calendar");
    let cal = CalendarRules::from_pack(&Pack::open(&dir).unwrap()).unwrap();
    let ymd = |t: &str| {
        let d = parse_dates(t, &cal);
        assert_eq!(d.len(), 1, "{t}");
        let r = d[0].readings[0].value;
        (r.year, r.month, r.day)
    };
    assert_eq!(ymd("le 3 mars 2026"), (Some(2026), Some(3), Some(3)));
    assert_eq!(ymd("le 1er janvier 2026"), (Some(2026), Some(1), Some(1)));
    assert_eq!(ymd("le 14 août 2026"), (Some(2026), Some(8), Some(14)));
    assert!(
        parse_dates("on 3 March 2026", &cal).is_empty(),
        "English month is not in the French pack"
    );
}
