# SemanticAnt

Comprehensive tool libraries covering the whole field of semantics and semantic analysis.

**Status: step 1 is built in this repository; only `semanticant-core` is on crates.io (0.1.0).** The crates `semanticant-text`, `-pack`, `-lexicon`,
`-value` and `-cues` (version 0.1.2 in this workspace) are NOT yet published; a published version is claimed here only after it is. The name
`semanticant` is reserved on crates.io at 0.0.0. Design and decisions: `docs/SPEC-STEP1.md`; changes: `CHANGELOG.md`; data packs: `packs/README.md`.

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
