#![allow(missing_docs)]
use semanticant_core::{LangTag, Span};
use semanticant_text::*;

fn en() -> LangTag {
    LangTag::parse("en").unwrap()
}
fn sp(a: usize, b: usize) -> Span {
    Span::new(a, b).unwrap()
}

#[test]
fn normalise_forms_and_offsets() {
    let text = "cafe\u{301} ok"; // e + combining acute
    let n = normalise(text, NormForm::Nfc);
    assert_eq!(n.text, "caf\u{e9} ok");
    // the 'é' (bytes 3..5 in output) came from original bytes 3..6 (e + U+0301)
    assert_eq!(n.original_span(sp(3, 5)), Some(sp(3, 6)));
    assert_eq!(n.original_span(sp(0, 3)), Some(sp(0, 3)));
    // a span touching two clusters maps to both
    assert_eq!(n.original_span(sp(2, 4)), Some(sp(2, 6)));
    assert_eq!(n.original_span(sp(0, 99)), None);
    assert_eq!(n.original_span(sp(2, 2)), None);
    let d = normalise("\u{e9}", NormForm::Nfd);
    assert_eq!(d.text, "e\u{301}");
    assert_eq!(d.original_span(sp(0, 3)), Some(sp(0, 2)));
    assert_eq!(normalise("\u{fb01}", NormForm::Nfc).text, "\u{fb01}");
    assert_eq!(normalise("\u{fb01}", NormForm::Nfkc).text, "fi");
    assert_eq!(normalise("\u{fb01}", NormForm::Nfkd).text, "fi");
}

#[test]
fn normalise_matches_whole_string_on_ordinary_text() {
    use unicode_normalization::UnicodeNormalization;
    for s in [
        "Ångström Å",
        "한국어 \u{1112}\u{1161}\u{11ab}",
        "naïve café résumé",
        "e\u{323}\u{301}",
        "",
        "plain ascii",
    ] {
        assert_eq!(
            normalise(s, NormForm::Nfc).text,
            s.nfc().collect::<String>(),
            "{s:?}"
        );
        assert_eq!(
            normalise(s, NormForm::Nfd).text,
            s.nfd().collect::<String>(),
            "{s:?}"
        );
    }
}

#[test]
fn fold_default_and_overrides() {
    let r = FoldRules::new();
    assert_eq!(fold("Straße ΟΔΟΣ ſ", &r), "strasse οδοσ s");
    assert_eq!(fold("HeLLo", &r), "hello");
    // Turkish tailoring is data, passed by the caller
    let tr = FoldRules::new()
        .with_override('İ', "i")
        .with_override('I', "ı");
    assert_eq!(fold("İI", &tr), "iı");
    assert_ne!(fold("I", &r), fold("I", &tr));
    let strip = FoldRules::new().strip_marks(true);
    assert_eq!(fold("Café Ångström", &strip), "cafe angstrom");
    assert_eq!(fold("Café", &r), "café");
}

#[test]
fn tokens_cover_text_exactly_with_kinds() {
    let seg = UaxSegmenter::new();
    let text = "Hello, world 42 \u{1f600} é!";
    let toks = seg.tokens(text, &en());
    let rebuilt: String = toks.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(rebuilt, text);
    for t in &toks {
        assert_eq!(t.span.slice(text), Some(t.text.as_str()));
    }
    let kinds: Vec<(&str, TokenKind)> = toks
        .iter()
        .filter(|t| t.kind != TokenKind::Whitespace)
        .map(|t| (t.text.as_str(), t.kind))
        .collect();
    assert_eq!(
        kinds,
        vec![
            ("Hello", TokenKind::Word),
            (",", TokenKind::Punct),
            ("world", TokenKind::Word),
            ("42", TokenKind::Number),
            ("\u{1f600}", TokenKind::Symbol),
            ("é", TokenKind::Word),
            ("!", TokenKind::Punct),
        ]
    );
}

#[test]
fn tokens_keep_contractions_and_decimals_per_uax29() {
    let seg = UaxSegmenter::new();
    let w: Vec<String> = seg
        .tokens("don't 3.14", &en())
        .into_iter()
        .filter(|t| t.kind != TokenKind::Whitespace)
        .map(|t| t.text)
        .collect();
    assert_eq!(w, vec!["don't", "3.14"]);
    assert!(seg.tokens("", &en()).is_empty());
}

fn sents(seg: &UaxSegmenter, text: &str) -> Vec<String> {
    seg.sentences(text, &en())
        .into_iter()
        .map(|v| v.value.slice(text).unwrap().to_string())
        .collect()
}

#[test]
fn sentences_pure_uax29_breaks_after_abbreviation() {
    let seg = UaxSegmenter::new();
    assert_eq!(
        sents(&seg, "Hello there. How are you? Fine!"),
        vec!["Hello there.", "How are you?", "Fine!"]
    );
    assert_eq!(
        sents(&seg, "I saw Dr. Smith today."),
        vec!["I saw Dr.", "Smith today."]
    );
}

#[test]
fn sentences_abbreviation_data_merges_and_explains() {
    let seg = UaxSegmenter::with_abbreviations(AbbreviationSet::from_entries(["Dr.", "e.g", "Mr"]));
    let text = "I saw Dr. Smith today. He said e.g. Mr. Jones left.";
    let v = seg.sentences(text, &en());
    let got: Vec<&str> = v.iter().map(|x| x.value.slice(text).unwrap()).collect();
    assert_eq!(
        got,
        vec!["I saw Dr. Smith today.", "He said e.g. Mr. Jones left."]
    );
    let first = &v[0];
    assert_eq!(first.reasons.len(), 1);
    assert_eq!(first.reasons[0].rule, "text.sentence.abbreviation");
    assert_eq!(first.reasons[0].span.unwrap().slice(text), Some("Dr"));
    // second sentence merged twice, reasons name each abbreviation
    let abbr: Vec<&str> = v[1]
        .reasons
        .iter()
        .map(|r| r.span.unwrap().slice(text).unwrap())
        .collect();
    assert_eq!(abbr, vec!["e.g", "Mr"]);
    // an unmerged sentence says so
    let plain = seg.sentences("One. Two.", &en());
    assert_eq!(plain[0].reasons[0].rule, "text.sentence.uax29");
}

#[test]
fn abbreviation_at_end_of_text_is_still_a_sentence_end() {
    let seg = UaxSegmenter::with_abbreviations(AbbreviationSet::from_entries(["etc"]));
    assert_eq!(sents(&seg, "Bring pens, etc."), vec!["Bring pens, etc."]);
    assert_eq!(sents(&seg, "   "), Vec::<String>::new());
    assert_eq!(sents(&seg, "A etc. b"), vec!["A etc. b"]);
}

#[test]
fn abbreviation_matching_is_case_insensitive_and_whole_word() {
    let seg = UaxSegmenter::with_abbreviations(AbbreviationSet::from_entries(["dr"]));
    assert_eq!(
        sents(&seg, "See DR. Smith now."),
        vec!["See DR. Smith now."]
    );
    // "ladr." must not match "dr"
    assert_eq!(
        sents(&seg, "He is a ladr. Smith left."),
        vec!["He is a ladr.", "Smith left."]
    );
}

#[test]
fn segmenter_is_object_safe() {
    let s: Box<dyn Segmenter> = Box::new(UaxSegmenter::new());
    assert_eq!(s.sentences("A. B.", &en()).len(), 2);
}

#[test]
fn only_a_dot_marks_an_abbreviation() {
    let seg = UaxSegmenter::with_abbreviations(AbbreviationSet::from_entries(["dr"]));
    assert_eq!(
        sents(&seg, "Wow, Dr! Smith left."),
        vec!["Wow, Dr!", "Smith left."]
    );
}
