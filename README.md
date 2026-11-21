# SemanticAnt

Comprehensive tool libraries covering the whole field of semantics and semantic analysis.

**Status: step 1 is built and all six crates are published on crates.io at 0.1.2 (2026-11-20, tag `v0.1.2`).** The name
`semanticant` is reserved at 0.0.0. Design and decisions: `docs/SPEC-STEP1.md`; changes: `CHANGELOG.md` (entries under "Unreleased" are on `main` but not yet published); data packs: `packs/README.md`.

| Crate | Does |
|---|---|
| `semanticant-core` | `Reason`, `Verdict`, `Span`, `LangTag`, `Provenance`, pack manifest types |
| `semanticant-text` | normalisation, case folding, tokens, sentences |
| `semanticant-pack` | loader for versioned, checksummed data packs |
| `semanticant-lexicon` | sense inventories; string, token-set, taxonomy and embedding-based similarity |
| `semanticant-value` | numbers, quantities and units, dates, times |
| `semanticant-cues` | negation, modality and hedge cues with scope |

Part of the field-library layer of the AnalysAnt family: these libraries are toolsets that AnalysAnt's lenses (and other projects, such as
SageAnt) use as dependencies. The standing rule (CireSnave, 2026-11-18): if a project needs a tool that falls in this field, the tool is
built here and used as a dependency by the project that needs it.

Licence: MIT OR Apache-2.0.
