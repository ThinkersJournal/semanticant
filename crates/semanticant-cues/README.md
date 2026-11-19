# semanticant-cues

Part of [SemanticAnt](https://github.com/ThinkersJournal/semanticant). Finds cue phrases for negation, modality and hedging in a token stream (from `semanticant-text`)
and reports each cue's SCOPE (what it applies to) with the reason the scope ended. Everything language-specific is data in a `kind = "cues"` pack:
cue phrases with kind, direction (forward/backward/none), window size and strength; scope terminators ("but", "however"); pseudo-cues that suppress a cue ("no increase");
sentence-end tokens. The engine contains no word of any language, so the same code reads English, German or Rust doc comments, depending on the pack.

Scope is a documented rule window, NegEx-style: from the cue, up to `window` tokens, stopping early at a terminator or a sentence-end token. It is a heuristic, not a parse:
it does not know syntax, so "no cough or fever" scopes both (as intended) and "no one came but we did" stops at "but" only if the pack lists it. Nothing here has been
evaluated against an annotated corpus (BioScope, NegEx test sets); treat the windows as defaults to tune.

Format and reason ids: `RULES.md`. Licence: MIT OR Apache-2.0.
