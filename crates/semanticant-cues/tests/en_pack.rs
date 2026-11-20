#![allow(missing_docs)]
//! The shipped English cue pack under `packs/en/cues` loads, verifies and finds cues.
use semanticant_core::LangTag;
use semanticant_cues::*;
use semanticant_pack::Pack;
use semanticant_text::{Segmenter, UaxSegmenter};
use std::path::PathBuf;

#[test]
fn english_cue_pack_works() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packs/en/cues");
    let rules = CueRules::from_pack(&Pack::open(&dir).unwrap()).unwrap();
    let text = "There is no evidence of fracture, but pain persists.";
    let toks = UaxSegmenter::new().tokens(text, &LangTag::parse("en").unwrap());
    let found = find_cues(&toks, &rules);
    let neg: Vec<_> = found
        .iter()
        .filter(|v| v.value.kind == CueKind::Negation)
        .collect();
    assert_eq!(neg.len(), 1);
    let scope = neg[0].value.scope.unwrap().slice(text).unwrap();
    assert!(
        scope.contains("fracture") && !scope.contains("pain"),
        "scope {scope:?} stops at `but`"
    );
    let t2 = "There was no change today.";
    let f2 = find_cues(&rules_toks(t2), &rules);
    assert!(
        !f2.is_empty() && f2.iter().all(|v| v.value.suppressed),
        "`no change` is a pseudo-cue"
    );
}

fn rules_toks(t: &str) -> Vec<semanticant_text::Token> {
    UaxSegmenter::new().tokens(t, &LangTag::parse("en").unwrap())
}

fn load(rel: &str) -> CueRules {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packs")
        .join(rel);
    CueRules::from_pack(&Pack::open(&dir).unwrap()).unwrap()
}

#[test]
fn german_and_rust_doc_packs_use_the_same_engine() {
    let de = load("de/cues");
    let t = "Es gibt keine Hinweise, aber Schmerzen bleiben.";
    let f = find_cues(&rules_toks(t), &de);
    let neg: Vec<_> = f
        .iter()
        .filter(|v| v.value.kind == CueKind::Negation)
        .collect();
    assert_eq!(neg.len(), 1);
    let scope = neg[0].value.scope.unwrap().slice(t).unwrap();
    assert!(
        scope.contains("Hinweise") && !scope.contains("Schmerzen"),
        "{scope:?}"
    );

    let rd = load("rust-doc/cues");
    let t = "This function must not be called twice, unless reset first.";
    let f = find_cues(&rules_toks(t), &rd);
    let neg: Vec<_> = f
        .iter()
        .filter(|v| v.value.kind == CueKind::Negation)
        .collect();
    assert_eq!(neg.len(), 1);
    let scope = neg[0].value.scope.unwrap().slice(t).unwrap();
    assert!(
        scope.contains("called") && !scope.contains("reset"),
        "{scope:?}"
    );
}
