#![allow(missing_docs)]
use semanticant_core::pack::{FileEntry, Manifest, SCHEMA_VERSION, Sha256Hex};
use semanticant_core::{LangTag, Provenance, Verdict};
use semanticant_cues::*;
use semanticant_pack::{Pack, sha256_hex};
use semanticant_text::{Segmenter, UaxSegmenter};
use std::collections::HashMap;

fn pack(kind: &str, files: &[(&str, &str)]) -> Pack {
    let m = Manifest {
        name: "t".into(),
        version: "1".into(),
        kind: kind.into(),
        schema: SCHEMA_VERSION,
        lang: None,
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

const EN: &str = "not\tnegation\tforward\t3\t1.0
no\tnegation\tforward\t4\t1.0
no evidence of\tnegation\tforward\t4\t1.0
without\tnegation\tforward\t3\t1.0
denies\tnegation\tforward\t4\t0.9
ruled out\tnegation\tbackward\t3\t0.9
may\tmodality\tforward\t2\t0.5
must\tmodality\tforward\t2\t1.0
suggests\thedge\tforward\t5\t0.6
probably\thedge\tforward\t3\t0.7
indeed\temphasis\tnone\t1\t0.3
";

fn en() -> CueRules {
    CueRules::from_pack(&pack(
        "cues",
        &[
            ("cues.tsv", EN),
            ("terminators.tsv", "but\nhowever\nalthough\n"),
            ("pseudo.tsv", "no increase\nnot only\n"),
        ],
    ))
    .unwrap()
}

/// (cue text, kind, scope text, suppressed)
fn run(text: &str, rules: &CueRules) -> Vec<(String, CueKind, Option<String>, bool)> {
    cues(text, rules)
        .into_iter()
        .map(|v| {
            (
                v.value.span.slice(text).unwrap().to_string(),
                v.value.kind,
                v.value.scope.map(|s| s.slice(text).unwrap().to_string()),
                v.value.suppressed,
            )
        })
        .collect()
}
fn cues(text: &str, rules: &CueRules) -> Vec<Verdict<Cue>> {
    let toks = UaxSegmenter::new().tokens(text, &LangTag::parse("und").unwrap());
    find_cues(&toks, rules)
}
fn rules_of(v: &Verdict<Cue>) -> Vec<&'static str> {
    v.reasons.iter().map(|r| r.rule).collect()
}

#[test]
fn negation_scope_stops_at_terminator() {
    let t = "The patient denies chest pain but reports cough.";
    assert_eq!(
        run(t, &en()),
        vec![(
            "denies".into(),
            CueKind::Negation,
            Some("chest pain".into()),
            false
        )]
    );
    assert_eq!(
        rules_of(&cues(t, &en())[0]),
        vec!["cues.cue.match", "cues.scope.terminator"]
    );
}

#[test]
fn longest_cue_wins_and_scope_ends_at_sentence_end() {
    let t = "There is no evidence of fever.";
    let r = run(t, &en());
    assert_eq!(
        r,
        vec![(
            "no evidence of".into(),
            CueKind::Negation,
            Some("fever".into()),
            false
        )],
        "not also a separate cue for the shorter 'no'"
    );
    assert_eq!(
        rules_of(&cues(t, &en())[0]),
        vec!["cues.cue.match", "cues.scope.sentence_end"]
    );
}

#[test]
fn window_limits_scope() {
    let t = "It is not good at all today";
    assert_eq!(
        run(t, &en()),
        vec![(
            "not".into(),
            CueKind::Negation,
            Some("good at all".into()),
            false
        )]
    );
    assert_eq!(
        rules_of(&cues(t, &en())[0]),
        vec!["cues.cue.match", "cues.scope.window"]
    );
}

#[test]
fn backward_scope_and_text_edge() {
    let t = "Cancer was ruled out.";
    let r = cues(t, &en());
    assert_eq!(r[0].value.span.slice(t), Some("ruled out"));
    assert_eq!(r[0].value.scope.unwrap().slice(t), Some("Cancer was"));
    assert_eq!(
        rules_of(&r[0]),
        vec!["cues.cue.match", "cues.scope.text_edge"]
    );
    let t2 = "Surgery done: the tumour, ruled out.";
    let r2 = cues(t2, &en());
    assert_eq!(
        r2[0].value.scope.unwrap().slice(t2),
        Some("the tumour,"),
        "a colon is not a sentence end"
    );
}

#[test]
fn backward_cue_at_the_start_has_no_scope() {
    let r = cues("Ruled out.", &en());
    assert_eq!(r[0].value.scope, None);
    assert_eq!(
        rules_of(&r[0]),
        vec!["cues.cue.match", "cues.scope.text_edge"]
    );
}

#[test]
fn empty_scope_is_reported_not_invented() {
    let r = cues("He did not.", &en());
    assert_eq!(r[0].value.scope, None);
    assert_eq!(rules_of(&r[0]), vec!["cues.cue.match", "cues.scope.empty"]);
    let r = cues("not but yes", &en());
    assert_eq!(r[0].value.scope, None);
}

#[test]
fn pseudo_cues_suppress_but_stay_visible() {
    let t = "There is no increase in pain.";
    let r = cues(t, &en());
    assert_eq!(r.len(), 1);
    assert!(r[0].value.suppressed);
    assert!(rules_of(&r[0]).contains(&"cues.cue.suppressed_pseudo"));
    let pseudo = r[0]
        .reasons
        .iter()
        .find(|x| x.rule == "cues.cue.suppressed_pseudo")
        .unwrap();
    assert_eq!(pseudo.span.unwrap().slice(t), Some("no increase"));
    let t2 = "It is not only cheap.";
    assert!(run(t2, &en())[0].3);
    // a cue that merely touches a pseudo phrase is not suppressed
    assert!(!run("no sign of increase", &en())[0].3);
    assert!(!run("not good", &en())[0].3);
}

#[test]
fn modality_hedge_and_other_kinds_with_strength() {
    let t = "It may not work, and it probably suggests a bug indeed.";
    let r = cues(t, &en());
    let kinds: Vec<(&str, CueKind, f32)> = r
        .iter()
        .map(|v| {
            (
                v.value.span.slice(t).unwrap(),
                v.value.kind.clone(),
                v.value.strength,
            )
        })
        .collect();
    assert_eq!(
        kinds,
        vec![
            ("may", CueKind::Modality, 0.5),
            ("not", CueKind::Negation, 1.0),
            ("probably", CueKind::Hedge, 0.7),
            ("suggests", CueKind::Hedge, 0.6),
            ("indeed", CueKind::Other("emphasis".into()), 0.3),
        ]
    );
    assert_eq!(r[0].value.scope.unwrap().slice(t), Some("not work"));
    assert_eq!(
        r[1].value.scope.unwrap().slice(t),
        Some("work, and"),
        "a comma is not a scope end: the 3-token window closes it"
    );
    assert_eq!(r[4].value.scope, None, "direction none");
}

#[test]
fn case_and_spacing_do_not_matter() {
    assert_eq!(
        run("NOT   Good", &en()),
        vec![("NOT".into(), CueKind::Negation, Some("Good".into()), false)]
    );
    assert_eq!(run("a  Ruled   Out", &en())[0].0, "Ruled   Out");
}

#[test]
fn no_cues_no_output() {
    assert!(run("Plain statement of fact.", &en()).is_empty());
    assert!(run("", &en()).is_empty());
    assert!(run("anything", &CueRules::new()).is_empty());
}

#[test]
fn the_engine_has_no_language_in_it() {
    // German
    let de = CueRules::from_pack(&pack(
        "cues",
        &[
            (
                "cues.tsv",
                "nicht\tnegation\tforward\t3\t1\nkein\tnegation\tforward\t3\t1\n",
            ),
            ("terminators.tsv", "aber\n"),
        ],
    ))
    .unwrap();
    assert_eq!(
        run("Er kommt nicht heute aber morgen.", &de),
        vec![(
            "nicht".into(),
            CueKind::Negation,
            Some("heute".into()),
            false
        )]
    );
    // Rust doc comments: a genre, not a language
    let docs = CueRules::from_pack(&pack(
        "cues",
        &[
            (
                "cues.tsv",
                "panics if\tpanic\tforward\t6\t1\nmust\tmodality\tforward\t4\t1\n",
            ),
            ("ends.tsv", "\u{3002}\n"),
        ],
    ))
    .unwrap();
    let t = "Panics if the caller must hold the lock. Then go.";
    let r = run(t, &docs);
    assert_eq!(
        r[0],
        (
            "Panics if".into(),
            CueKind::Other("panic".into()),
            Some("the caller must hold the lock".into()),
            false
        ),
        "window 6 closes the scope"
    );
    assert_eq!(
        r[1],
        (
            "must".into(),
            CueKind::Modality,
            Some("hold the lock.".into()),
            false
        ),
        "'.' is not an end in this pack, so it is inside the scope"
    );
    // custom ends: "." no longer stops scope, the ideographic full stop does
    let r = run("must do it. now\u{3002} later", &docs);
    assert_eq!(r[0].2.as_deref(), Some("do it. now"));
}

#[test]
fn rules_can_be_built_in_code() {
    let r = CueRules::new()
        .with_cue("never", CueKind::Negation, Direction::Forward, 2, 2.0)
        .with_terminator("YET")
        .with_ends(&["|"]);
    let t = "never ever yet more";
    let c = cues(t, &r);
    assert_eq!(c[0].value.scope.unwrap().slice(t), Some("ever"));
    assert_eq!(c[0].value.strength, 1.0, "strength is clamped to 1");
    let none = CueRules::new().with_cue("", CueKind::Hedge, Direction::None, 1, 1.0);
    assert!(cues("x", &none).is_empty(), "an empty phrase never matches");
}

#[test]
fn pack_validation() {
    let bad = |row: &str| CueRules::from_pack(&pack("cues", &[("cues.tsv", row)])).unwrap_err();
    assert!(matches!(
        bad("not\tnegation\tforward\t3\n"),
        CueError::BadRow("cues.tsv", 1, _)
    ));
    assert!(matches!(
        bad("not\tnegation\tsideways\t3\t1\n"),
        CueError::BadRow(..)
    ));
    assert!(matches!(
        bad("not\tnegation\tforward\t0\t1\n"),
        CueError::BadRow(..)
    ));
    assert!(matches!(
        bad("not\tnegation\tforward\tx\t1\n"),
        CueError::BadRow(..)
    ));
    assert!(matches!(
        bad("not\tnegation\tforward\t3\t1.5\n"),
        CueError::BadRow(..)
    ));
    assert!(matches!(
        bad(" \tnegation\tforward\t3\t1\n"),
        CueError::BadRow(..)
    ));
    assert!(matches!(
        CueRules::from_pack(&pack("units", &[("cues.tsv", EN)])),
        Err(CueError::WrongKind(_))
    ));
    assert!(matches!(
        CueRules::from_pack(&pack("cues", &[("x.tsv", "a")])),
        Err(CueError::MissingFile("cues.tsv"))
    ));
}

#[test]
fn a_pseudo_phrase_must_cover_the_whole_cue() {
    let r = en().with_pseudo("sign no");
    // "sign no" overlaps the cue "no evidence of" only at its first token: not wholly inside, so not suppressed
    let t = "sign no evidence of fever";
    let v = run(t, &r);
    assert_eq!(v.len(), 1);
    assert!(!v[0].3);
    // but a pseudo phrase that does cover it suppresses
    assert!(
        run(
            "sign no evidence of fever",
            &en().with_pseudo("sign no evidence of fever")
        )[0]
        .3
    );
}

#[test]
fn a_cue_consumes_its_own_tokens() {
    let r = CueRules::new()
        .with_cue("not only", CueKind::Negation, Direction::Forward, 2, 1.0)
        .with_cue("only", CueKind::Hedge, Direction::Forward, 2, 1.0);
    let v = run("it is not only cheap", &r);
    assert_eq!(v.len(), 1, "{v:?}");
    assert_eq!(v[0].0, "not only");
}

#[test]
fn a_zero_window_still_covers_one_token() {
    let r = CueRules::new().with_cue("never", CueKind::Negation, Direction::Forward, 0, 1.0);
    assert_eq!(run("never ever more", &r)[0].2.as_deref(), Some("ever"));
}
