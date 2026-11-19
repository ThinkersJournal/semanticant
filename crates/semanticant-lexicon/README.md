# semanticant-lexicon

Part of [SemanticAnt](https://github.com/ThinkersJournal/semanticant).

* `SenseInventory`: the interface to any sense inventory (lemmas -> senses, typed relations, glosses). `InMemoryInventory` loads one from a data pack
  (`kind = "lexicon"`, files `senses.tsv` and `relations.tsv`, see `RULES.md`). `SenseId` is opaque and local to an inventory; global identity is not this crate's job.
* `Measure<T>`: `score(a, b) -> Verdict<f64>`. Implemented for strings (Levenshtein, Jaro-Winkler, Dice bigrams), token sets (Jaccard, overlap, containment),
  taxonomies (path, Wu-Palmer, Resnik, Lin and Jiang-Conrath with intrinsic information content) and vectors from a caller-supplied `Embedder` (cosine). No model ships here.
* Scores are in `[0, 1]` (Resnik is in raw IC units; see its docs) and carry the reasons for them.

Standard library plus `semanticant-core` and `semanticant-pack`. Licence: MIT OR Apache-2.0.
