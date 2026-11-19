# semanticant-value

Part of [SemanticAnt](https://github.com/ThinkersJournal/semanticant). Turns text into canonical values, driven by data (packs), never by a hard-coded language:

* `parse_numbers`: digits (sign, group separators, decimals) and spelled-out numbers from a word list (`NumberRules`). Values are `rust_decimal::Decimal`.
* `parse_quantities` / `convert`: a number followed by a unit surface from a `UnitTable`; conversion uses exact decimal factors (and offsets for temperature).
* `parse_dates` / `parse_times`: dates that may be partial (never padded), ambiguous `03/04/2026` returns BOTH readings with reasons; calendar validity is checked with `jiff`.

Every result carries `Reason`s (ids in `RULES.md`). Not covered yet: time zones and offsets, fractions and ranges ("1/2", "3-5"), two-digit years, multipliers like "3 million",
compound units ("km/h" is just a surface form if the pack lists it). Numbers beyond `Decimal`'s range are skipped, not clamped.

Pack formats: `number-words` (`words.tsv`), `units` (`units.tsv`), `calendar` (`months.tsv`, optional `ordinals.tsv`, `meridiem.tsv`); see `RULES.md`.

Licence: MIT OR Apache-2.0.
