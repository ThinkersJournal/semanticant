# semanticant-text

Part of [SemanticAnt](https://github.com/ThinkersJournal/semanticant). Unicode normalisation (with an offset map back to the original),
case folding, word tokenisation and sentence segmentation. Every span is a byte range into the ORIGINAL text.

* No language is hard-coded: language-specific behaviour (sentence-final abbreviations, Turkic case rules) is data the caller passes in.
* Tokenisation and sentence boundaries follow UAX #29 via `unicode-segmentation`; abbreviation handling is a data-driven merge pass that records a `Reason`.
* The `Segmenter` trait lets another segmenter (for example an ICU-based one for CJK/Thai) be dropped in.

Rule ids are in `RULES.md`. Licence: MIT OR Apache-2.0.
