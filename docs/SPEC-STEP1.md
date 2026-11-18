# SemanticAnt: SPEC-STEP1 (lane draft of 2026-11-18, docs only, no code)

Expands `SPEC-SEED-2026-11-18.md`. Marks: **[said]** CireSnave, **[PM]** PM seed, **[lane]** this lane's proposal, **[unverified]** not checked.
Section 9 lists exactly what was and was not read. Nothing here is built.

## 1. Scope and non-goals
In scope (seed section 2, unchanged): lexical semantics, propositional and compositional meaning, sentence-level entailment/contradiction/presupposition,
similarity and relatedness. Step 1 builds only the four foundation tools of section 3.
Non-goals for step 1: any model or embedding inside the core; parsing or POS tagging (a caller or later tool supplies it); hard-coded language data;
ontology, identity or evidence machinery (sibling projects); a lens or MCP surface (AnalysAnt).

## 2. Principles that bind the API
Generic over any use of the tool type [said 2026-11-18]: consumers supply test cases, never API shape. Deterministic, explainable (every verdict carries
a `Reason` list), language as data, `#![forbid(unsafe_code)]`, permissive deps only, one version for the whole project.

## 3. The four foundation tools (confirmed with one correction)
The seed's four stand. **Correction [lane]:** tool 4 bundles two unrelated things (value parsing; cue detection with scope). Ship them as two crates
(`-value`, `-cues`) so a consumer needing only dates does not take cue rule data. Still one PR per day, same tools.

| Crate | Tool | Core dependencies (all permissive) |
|---|---|---|
| `semanticant-core` | `Reason`, `Span`, `LangTag`, `Provenance`, error types; std only | none |
| `semanticant-text` | normalise, fold, tokenise, segment | `unicode-normalization`, `unicode-segmentation` |
| `semanticant-pack` | versioned, checksummed data packs + loader | `sha2` (or `blake3`), a TOML reader |
| `semanticant-lexicon` | sense inventory trait + similarity/relatedness + embedding trait | core, pack, text |
| `semanticant-value` | numbers, units, quantities, dates/times to canonical values | `rust_decimal`, `jiff` |
| `semanticant-cues` | negation, modality, hedge cues with scope | core, text, pack |

### 3.1 core (sketch)
```rust
pub struct Span { pub start: usize, pub end: usize }          // byte offsets into the ORIGINAL text
pub struct LangTag(String);                                   // BCP 47, validated, never defaulted to English
pub struct Reason { pub rule: &'static str, pub span: Option<Span>, pub note: String }
pub struct Verdict<T> { pub value: T, pub reasons: Vec<Reason> }
```
### 3.2 text
```rust
pub trait Segmenter { fn tokens(&self, text: &str, lang: &LangTag) -> Vec<Token>;
                      fn sentences(&self, text: &str, lang: &LangTag) -> Vec<Span>; }
pub struct Token { pub span: Span, pub text: String, pub kind: TokenKind }
pub fn normalise(text: &str, form: NormForm) -> Normalised;   // keeps an offset map back to the original
pub fn fold(text: &str, lang: &LangTag, rules: &FoldRules) -> String;
```
Default segmenter = UAX #29 via `unicode-segmentation`; per-language abbreviation and exception lists are a data pack, so "Dr." handling is data.
Offsets always map to the original text so a caller can quote evidence.
### 3.3 pack
A pack is a directory or archive: `pack.toml` (name, version, kind, `lang`, schema version, `licence` as an SPDX expression, `source` with URL and credit,
file list with SHA-256 each) plus data files. Loader verifies every checksum and refuses a pack with no licence or source. Format of data files: **[ASK]** (section 8).
```rust
pub struct Pack { pub manifest: Manifest, /* files by name */ }
impl Pack { pub fn open(path: &Path) -> Result<Pack, PackError>; pub fn file(&self, name: &str) -> Option<&[u8]>; }
```
### 3.4 lexicon and similarity
```rust
pub trait SenseInventory { fn senses(&self, lemma: &str, lang: &LangTag) -> Vec<SenseId>;
    fn relations(&self, s: SenseId, rel: Relation) -> Vec<SenseId>;      // synonym, antonym, hypernym, hyponym, ...
    fn gloss(&self, s: SenseId) -> Option<&str>; }
pub trait Measure<T> { fn score(&self, a: &T, b: &T) -> Verdict<f64>; }   // string, token-set, taxonomy impls
pub trait Embedder { fn embed(&self, text: &str) -> Vec<f32>; }           // caller supplies; core never ships one
```
Measures in step 1: Levenshtein/Jaro-Winkler/Dice on strings; Jaccard/overlap/containment on token sets; taxonomy path, Wu-Palmer, and information-content
(Resnik, Lin) over any `SenseInventory` that exposes hypernym edges; cosine over any `Embedder`. `SenseId` is opaque and local to an inventory; global
identity belongs to OntologicAnt (section 7).
### 3.5 value
```rust
pub struct Quantity { pub value: Decimal, pub unit: Option<UnitId>, pub span: Span }
pub fn parse_numbers(text: &str, lang: &LangTag, rules: &NumberRules) -> Vec<Verdict<Decimal>>;   // digits, grouping, spelled-out, fractions, ranges
pub fn parse_quantities(...) -> Vec<Verdict<Quantity>>;  pub fn parse_datetimes(...) -> Vec<Verdict<DateTimeValue>>;
```
Number words, unit tables and date formats are packs. Unit conversion is a table of exact rational factors, no float drift. Ambiguity ("03/04/05")
returns every reading with a reason, never silently picks one. Partial dates stay partial (`2026-11`), not padded.
### 3.6 cues
```rust
pub struct Cue { pub kind: CueKind /* Negation|Modality|Hedge|... */, pub span: Span, pub scope: Option<Span>, pub strength: f32 }
pub fn find_cues(tokens: &[Token], rules: &CueRules) -> Vec<Verdict<Cue>>;
```
Cue lists and scope-terminating words are packs; scope is by documented rule windows (a NegEx-style approach, section 4), reported with the rule that
closed it. English and Rust-doc-comment pack are the first two, so the engine is shown not to be English-only.

## 4. Survey: what exists (reuse over rebuild)
Metadata (version, licence, MSRV, downloads) from the crates.io API on 2026-11-18; "read" means the README/metadata only, not the source. **[unverified]**
anywhere I did not read code.
- **Reuse as dependencies:** `unicode-normalization` 0.1.26 and `unicode-segmentation` 1.13.3 (MIT OR Apache-2.0; ~690M downloads each; UAX #15/#29);
  `jiff` 1.0.1 (Unlicense OR MIT; dates and time zones: do not write calendar code); `rust_decimal` 1.43.0 (MIT); `sha2` 0.11.1 (MIT OR Apache-2.0).
- **Evaluate, likely reuse:** `strsim` 0.11.1 (MIT; Levenshtein, Jaro-Winkler, Dice; but last released 2024-04 and returns bare numbers, so wrap it behind `Measure` rather than expose it);
  `icu_segmenter`/`icu_casemap` 2.4.0 (Unicode-3.0; real CJK/Thai segmentation and locale case mapping; heavier, so offer as an optional segmenter behind the same trait).
- **Reject or only a source of ideas:** `caseless` 0.2.2 (MIT; full-fold only, no locale rules); `rust-stemmers` (Snowball; last release 2019; a stemmer is lossy and not a
  lemmatiser, so not a foundation tool); `nlprule` 0.6.4 (last release 2021; carries LanguageTool-derived rules whose licence must be checked before any reuse);
  `whatlang`/`lingua` (language identification: useful to a caller, outside step 1); `fend-core` (a calculator; too heavy and not a parser of text);
  `uom` (typed compile-time units; not text parsing, so not the unit table, though its unit definitions' licence is Apache-2.0 OR MIT if we want to cross-check values);
  `dateparser`/`chrono-english`/`human-date-parser` (English-first, hard-coded; not data-driven).
- **Gap:** no maintained, data-driven Rust crate found for WordNet-style sense inventories (`wordnet-db` 0.1.3 is a memory-mapped reader, 5k downloads, single maintainer; a
  possible *loader target* for a pack, not a dependency), for number-word/quantity extraction across languages, or for negation/modality cue scope. This is where we build.
- **Data and literature (known to me, not read in this session):** Princeton WordNet (permissive licence that requires its notice kept), Open Multilingual Wordnet (per-language licences,
  some CC BY), Wiktionary-derived data (CC BY-SA: share-alike, so any pack from it carries that licence, and a pack licence field must be able to say so), Unicode CLDR (Unicode licence),
  NegEx/ConText (rule-and-window cue scope, Chapman et al.), BioScope and FactBank (annotated corpora for cue scope and event factuality), Resnik 1995, Lin 1998, Wu-Palmer 1994, Jiang-Conrath 1997
  (taxonomy similarity). Every one needs to be read, and each licence checked, before a pack or citation is claimed. None is vendored.

## 5. Consumer seams
- **AnalysAnt** (`analysant-lexical`, `analysant-semantic`): the intended first consumer. It would depend on `-text`, `-lexicon`, `-cues`, `-value` and write its lenses over `Verdict`. **When it needs
  each tool is not known to this lane [ASK PM]**; the build order in section 6 is therefore by dependency, not by consumer date.
- **SageAnt**: `-lexicon` measures and the `Embedder` trait (its own embedder plugs in); `-text` segmentation for chunking.
- Seam rule: a consumer's needs become tests in this repo (a fixture with the consumer's sentence and the expected reason trace), never a flag or type named for the consumer.

## 6. Build order (one PR per day, red-first tests with sabotage)
Day 1 `core` + `text`; day 2 `pack`; day 3 `lexicon` (traits, string/token-set measures, an in-memory inventory, a test pack); day 4 `value`; day 5 `cues`. Each PR: tests written failing first, then
a deliberate mutation of the implementation shown to turn them red, no `unsafe`, docs claims checked against code, version allocated by the PM. Spec PR first, then Day 1 after it merges.

## 7. Shared substrate with the two siblings (settle, do not triplicate)
Proposal [lane], consistent with the PM's dependency order SemanticAnt <- OntologicAnt <- EpistemologicAnt:
`semanticant-core` owns `Span`, `LangTag`, `Reason`, `Verdict`, `Provenance` (source, licence, credit, retrieval date) and the pack format, because text and packs sit at the bottom.
Identifiers are **not** here: `SenseId` stays local to an inventory and OntologicAnt owns stable global identity. A name here for a thing OntologicAnt later names differently is cheap to bridge;
a stable-ID scheme invented here is not cheap to undo. Needs the sibling specs [unverified: not yet read].

## 8. Decisions needed [ASK]
1. Pack data-file format: TOML manifest + JSON Lines/TSV data (readable, diffable, no custom parser) vs a binary index. **Recommend** text first, a binary cache derived from it later.
2. Split tool 4 into `-value` and `-cues` (section 3). **Recommend** yes.
3. `Reason.rule` is a static id and `note` is free text in English; is explanation text itself translatable data? **Recommend** ids stable and documented, notes English-only in step 1.
4. First consumer dates (to PM): when does AnalysAnt need which tool?
5. MSRV: the dependencies need 1.85-1.88 (`sha2` 1.88). **Recommend** MSRV 1.88, permitted by the standing rule (2026-10-08).

## 9. What is unverified
Read: crates.io metadata for the 21 crates in section 4 (the survey's only reading). **Not read:** every crate's source and README beyond the description line, every paper and corpus named in section 4,
any WordNet licence text, the two sibling specs, `AnalysAnt`'s code (so no seam is checked against its real types). The API sketches have not been compiled. Download counts and dates are as of 2026-11-18 and
only say popularity and last release, not quality.

— lane: semanticant
