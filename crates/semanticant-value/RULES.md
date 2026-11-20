# Pack formats and reason ids

`number-words` / `words.tsv`: `word<TAB>unit|tens|scale|connector<TAB>value` (unit 0-19, tens 20-99, scale >= 100; connector has no value).
`units` / `units.tsv`: `surface<TAB>id<TAB>dimension<TAB>factor[<TAB>offset]`; `base = value * factor + offset`; several surfaces may share an id and must agree.
`calendar` / `months.tsv`: `name<TAB>1..12`; optional `ordinals.tsv` (one suffix per row) and `meridiem.tsv` (`word<TAB>am|pm`).
Blank lines and `#` lines are skipped.

Reason ids (stable): `value.number.digits`, `value.number.words`, `value.quantity`, `value.unit.convert`, `value.date.iso`, `value.date.numeric.dmy`,
`value.date.numeric.mdy`, `value.date.ambiguous`, `value.date.textual`, `value.time.24h`, `value.time.12h`.
