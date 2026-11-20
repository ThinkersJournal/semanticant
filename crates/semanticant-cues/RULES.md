# Pack format and reason ids

`kind = "cues"`. Blank lines and lines starting with `#` are skipped (the pack loader's rule), so a phrase cannot START with `#`.
* `cues.tsv` (required): `phrase<TAB>kind<TAB>direction<TAB>window<TAB>strength`. `phrase` is space-separated lowercase words/punctuation tokens as the tokenizer yields them;
  `kind` is `negation|modality|hedge` or any other word (kept as `Other`); `direction` is `forward|backward|none`; `window` is a whole number >= 1 (tokens); `strength` is 0..=1.
* `terminators.tsv` (optional): one lowercase token per row; scope stops BEFORE it.
* `pseudo.tsv` (optional): one phrase per row; a cue wholly inside a pseudo phrase is reported with `suppressed = true`.
* `ends.tsv` (optional): sentence-end tokens, one per row; replaces the default (`. ! ? ; 。 ！ ？`).

Reason ids (stable): `cues.cue.match`, `cues.scope.window`, `cues.scope.terminator`, `cues.scope.sentence_end`, `cues.scope.text_edge`, `cues.scope.empty`, `cues.cue.suppressed_pseudo`.
