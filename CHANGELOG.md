# Changelog

One version for the whole project. Pre-1.0: a breaking change bumps the second number.

## 0.1.2
- Added `semanticant-value`: numbers (digits and spelled-out), quantities with units and exact conversion, partial dates (ambiguous numeric dates return every reading) and times of day, all driven by packs, every result with reasons.

## 0.1.1
- Added `semanticant-text`: normalisation with an offset map, case folding, UAX #29 tokens and sentences with a data-driven abbreviation merge.
- Added `semanticant-pack`: `pack.toml` parser and loader that verifies every file's SHA-256 and refuses packs without licence and source.
- Added `semanticant-lexicon`: `SenseInventory` + pack-backed `InMemoryInventory`; string, token-set, taxonomy (path, Wu-Palmer, Resnik, Lin) and cosine-over-caller-embedder measures, every score with reasons.
- Added `deny.toml` and the `deny` CI job.

## 0.1.0
- First release: `semanticant-core` (`Span`, `LangTag`, `Reason`, `Verdict`, `Provenance`, pack `Manifest`).
