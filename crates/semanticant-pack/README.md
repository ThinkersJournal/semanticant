# semanticant-pack

Loads [SemanticAnt](https://github.com/ThinkersJournal/semanticant) data packs. A pack is a directory with `pack.toml` and the data files it lists
(JSON Lines or TSV text in step 1). The loader parses the manifest, validates it (`semanticant-core::pack::Manifest::validate`: licence, source and
credit required; no path escapes), reads each listed file and verifies its SHA-256 before handing out any bytes. Unlisted files in the directory are ignored; symlinks are refused.

```toml
schema = 1
name = "demo-abbreviations"
version = "1.0.0"
kind = "abbreviations"
lang = "en"

[provenance]
source = "Hand-written example"
licence = "MIT OR Apache-2.0"
credit = "SemanticAnt authors"
retrieved = "2026-11-18"

[[files]]
name = "abbr.tsv"
sha256 = "<64 lowercase hex>"
```

Licence: MIT OR Apache-2.0.
