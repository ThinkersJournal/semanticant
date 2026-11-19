# Packs

Data-only language packs loaded by `semanticant-pack`. Layout: `packs/<lang>/<name>/` with a
`pack.toml` (metadata and checksums) plus tab-separated data files.

| Pack | Used by |
|------|---------|
| `en/number-words`, `en/units`, `en/calendar` | `semanticant-value` |
| `de/calendar` | `semanticant-value` |
| `en/cues`, `de/cues`, `rust-doc/cues` | `semanticant-cues` |

Rules: a phrase in a pack cannot start with `#` (comment marker). Packs are byte-exact
(`.gitattributes` disables line-ending conversion); editing a data file requires updating the
checksum in its `pack.toml`, and the load tests fail otherwise.
