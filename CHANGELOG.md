# Changelog

One version for the whole project. Pre-1.0: a breaking change bumps the second number.

## 0.1.3

- `easyants-card.toml` and `examples/quickstart.rs` (built by CI through the unpublished `semanticant-examples` crate); README states the real publish status.
- Added `LeacockChodorow` (normalised to [0, 1]) to `semanticant-lexicon`'s taxonomy measures.

## 0.1.2

- `packs/en/`: self-authored English packs (number-words, units, calendar, cues) with real checksums; `.gitattributes` keeps them byte-exact. Tests load them, and a flipped byte is refused.
- Added `semanticant-cues`: negation, modality and hedge cues with scope windows, terminators and pseudo-cue suppression, all from packs; each cue carries the reason its scope ended.
- Added `JiangConrath` (intrinsic-IC distance, scored `1/(1+d)`) to `semanticant-lexicon`'s taxonomy measures.
- Added `semanticant-value`: numbers (digits and spelled-out), quantities with units and exact conversion, partial dates (ambiguous numeric dates return every reading) and times of day, all driven by packs, every result with reasons.

## 0.1.1
- Added `semanticant-text`: normalisation with an offset map, case folding, UAX #29 tokens and sentences with a data-driven abbreviation merge.
- Added `semanticant-pack`: `pack.toml` parser and loader that verifies every file's SHA-256 and refuses packs without licence and source.
- Added `semanticant-lexicon`: `SenseInventory` + pack-backed `InMemoryInventory`; string, token-set, taxonomy (path, Wu-Palmer, Resnik, Lin) and cosine-over-caller-embedder measures, every score with reasons.
- Added `deny.toml` and the `deny` CI job.

## 0.1.0
- First release: `semanticant-core` (`Span`, `LangTag`, `Reason`, `Verdict`, `Provenance`, pack `Manifest`).
