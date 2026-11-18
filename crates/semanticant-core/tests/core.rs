#![allow(missing_docs)]
use semanticant_core::pack::{FileEntry, Manifest, SCHEMA_VERSION, Sha256Hex};
use semanticant_core::*;

const H: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn prov() -> Provenance {
    Provenance {
        source: "Example Wordnet".into(),
        licence: "MIT OR Apache-2.0".into(),
        credit: "Example authors".into(),
        url: None,
        retrieved: Some("2026-11-18".into()),
    }
}
fn manifest() -> Manifest {
    Manifest {
        name: "demo".into(),
        version: "1.0.0".into(),
        kind: "lexicon".into(),
        schema: SCHEMA_VERSION,
        lang: Some(LangTag::parse("en").unwrap()),
        provenance: prov(),
        files: vec![FileEntry {
            name: "data/senses.jsonl".into(),
            sha256: Sha256Hex::parse(H).unwrap(),
        }],
    }
}

#[test]
fn span_rejects_reversed_and_slices_text() {
    assert_eq!(
        Span::new(3, 2),
        Err(CoreError::InvalidSpan { start: 3, end: 2 })
    );
    let s = Span::new(1, 4).unwrap();
    assert_eq!((s.len(), s.is_empty()), (3, false));
    assert_eq!(s.slice("abcdef"), Some("bcd"));
    assert_eq!(Span::new(0, 99).unwrap().slice("abc"), None);
    // not on a char boundary: 'é' is two bytes
    assert_eq!(Span::new(0, 1).unwrap().slice("é"), None);
}

#[test]
fn span_relations() {
    let a = Span::new(0, 5).unwrap();
    let b = Span::new(2, 4).unwrap();
    let c = Span::new(5, 8).unwrap();
    assert!(a.contains(&b) && !b.contains(&a));
    assert!(a.overlaps(&b));
    assert!(!a.overlaps(&c), "half-open: touching spans do not overlap");
    assert_eq!(a.union(&c), Span::new(0, 8).unwrap());
    assert!(Span::new(2, 2).unwrap().is_empty());
}

#[test]
fn langtag_canonicalises_and_rejects() {
    for (i, o) in [
        ("EN", "en"),
        ("en-gb", "en-GB"),
        ("ZH-hant-tw", "zh-Hant-TW"),
        ("und", "und"),
        ("sr-Latn", "sr-Latn"),
    ] {
        assert_eq!(LangTag::parse(i).unwrap().as_str(), o);
    }
    assert_eq!(LangTag::parse("zh-Hant-TW").unwrap().language(), "zh");
    for bad in [
        "",
        "e",
        "en_GB",
        "en-",
        "-en",
        "english1",
        "en--GB",
        "e1",
        "en-toolongsubtag",
        "ünï",
    ] {
        assert!(LangTag::parse(bad).is_err(), "{bad:?} should be rejected");
    }
    assert!("fr".parse::<LangTag>().is_ok());
}

#[test]
fn verdict_keeps_reasons_in_order() {
    let r1 = Reason::new("a.first", "one");
    let r2 = Reason::at("a.second", Span::new(0, 1).unwrap(), "two");
    let v = Verdict::new(1)
        .because(r1.clone())
        .and_then(|n| Verdict::new(n + 1).because(r2.clone()));
    assert_eq!(v.value, 2);
    assert_eq!(v.reasons, vec![r1, r2]);
    assert_eq!(v.map(|n| n * 10).reasons.len(), 2);
}

#[test]
fn provenance_requires_source_licence_credit() {
    assert!(prov().validate().is_ok());
    for f in [
        |p: &mut Provenance| p.source = " ".into(),
        |p: &mut Provenance| p.licence = "".into(),
        |p: &mut Provenance| p.credit = "".into(),
    ] {
        let mut p = prov();
        f(&mut p);
        assert!(matches!(p.validate(), Err(CoreError::EmptyField(_))));
    }
}

#[test]
fn licence_shape() {
    for ok in [
        "MIT",
        "CC-BY-SA-4.0",
        "MIT OR Apache-2.0",
        "(MIT OR Apache-2.0) AND Unicode-3.0",
        "Apache-2.0 WITH LLVM-exception",
        "GPL-2.0+",
    ] {
        let mut p = prov();
        p.licence = ok.into();
        assert!(p.validate().is_ok(), "{ok}");
    }
    for bad in [
        "OR MIT",
        "MIT OR",
        "MIT AND AND Apache-2.0",
        "(MIT",
        "MIT)",
        "MIT Apache-2.0",
        "M!T",
        "()",
    ] {
        let mut p = prov();
        p.licence = bad.into();
        assert!(
            matches!(p.validate(), Err(CoreError::InvalidLicence(_))),
            "{bad}"
        );
    }
}

#[test]
fn dates() {
    for bad in [
        "2026-13-01",
        "2026-00-10",
        "2026-11-32",
        "26-11-18",
        "2026/11/18",
        "2026-11-1a",
    ] {
        let mut p = prov();
        p.retrieved = Some(bad.into());
        assert!(
            matches!(p.validate(), Err(CoreError::InvalidDate(_))),
            "{bad}"
        );
    }
}

#[test]
fn checksum_is_64_lowercase_hex() {
    assert!(Sha256Hex::parse(H).is_ok());
    assert!(Sha256Hex::parse(&H.to_uppercase()).is_err());
    assert!(Sha256Hex::parse(&H[..63]).is_err());
    assert!(Sha256Hex::parse(&format!("{H}0")).is_err());
    assert!(Sha256Hex::parse(&H.replace('0', "g")).is_err());
}

#[test]
fn manifest_valid_and_each_defect_refused() {
    assert!(manifest().validate().is_ok());
    let mut m = manifest();
    m.schema = 99;
    assert_eq!(m.validate(), Err(CoreError::UnsupportedSchema(99)));
    let mut m = manifest();
    m.provenance.licence = String::new();
    assert!(m.validate().is_err(), "a pack with no licence is refused");
    let mut m = manifest();
    m.files.clear();
    assert_eq!(m.validate(), Err(CoreError::EmptyField("files")));
    let mut m = manifest();
    m.kind = " ".into();
    assert_eq!(m.validate(), Err(CoreError::EmptyField("kind")));
    let mut m = manifest();
    m.files.push(m.files[0].clone());
    assert!(matches!(m.validate(), Err(CoreError::DuplicateFile(_))));
}

#[test]
fn manifest_file_names_cannot_escape() {
    for bad in [
        "",
        "/etc/passwd",
        "../x",
        "a/../b",
        "a//b",
        "./a",
        r"a\b",
        "C:/x",
        "a/",
        "a\0b",
    ] {
        let mut m = manifest();
        m.files[0].name = bad.into();
        assert!(
            matches!(m.validate(), Err(CoreError::InvalidFileName(_))),
            "{bad:?}"
        );
    }
}

#[test]
fn errors_display_name_the_input() {
    assert!(
        CoreError::InvalidLangTag("x_y".into())
            .to_string()
            .contains("x_y")
    );
    let e: Box<dyn std::error::Error> = Box::new(CoreError::UnsupportedSchema(7));
    assert!(e.to_string().contains('7'));
}
