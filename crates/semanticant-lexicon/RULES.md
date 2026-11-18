# Pack format and reason ids

## Lexicon pack (`kind = "lexicon"`), text first
`senses.tsv`: `key<TAB>lang<TAB>lemmas<TAB>gloss` (lemmas separated by `|`; gloss may be empty). `key` is unique inside the pack.
`relations.tsv`: `from_key<TAB>relation<TAB>to_key`. `relation` is `synonym|antonym|hypernym|hyponym|meronym|holonym|related`, or any other word (kept as `Other`).
Each `hypernym` row also yields the inverse `hyponym` (and vice versa) automatically. Blank lines and `#` lines are skipped.

## Reason ids (stable)
`lexicon.string.levenshtein`, `lexicon.string.jaro_winkler`, `lexicon.string.dice`, `lexicon.tokens.jaccard`, `lexicon.tokens.overlap`, `lexicon.tokens.containment`,
`lexicon.taxonomy.path`, `lexicon.taxonomy.wu_palmer`, `lexicon.taxonomy.resnik`, `lexicon.taxonomy.lin`, `lexicon.taxonomy.no_common_ancestor`,
`lexicon.cosine`, `lexicon.cosine.zero_vector`, `lexicon.cosine.dimension_mismatch`.
