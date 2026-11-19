# HANDOFF: semanticant

You own `C:\Projects\semanticant` (github.com/ThinkersJournal/semanticant, public). Read `C:\Projects\CLAUDE.md`, `docs/SPEC-STEP1.md`.
Sonnet only; never Opus. Primary stays on `main`; branches in worktrees (`C:\Projects\semanticant-wt-<topic>`). One PR per project per UTC day.

## State (2026-11-19)
Main is at 0.1.1: `-core` (published 0.1.0), `-text`, `-pack`, `-lexicon` (merged #3, NOT published: the PM publishes when a consumer asks). `deny` is required in protection.
Deviations from SPEC 3.2: `fold(text, rules)` has no lang arg; `sentences` returns `Vec<Verdict<Span>>`; `-value` returns dates and times as separate matches (no combined DateTimeValue; callers join by span).
Worktree `C:\Projects\semanticant-wt-value`, branch `value-day4` (pushed, NO PR): `semanticant-value` (13 tests, ~20 mutations red) and `semanticant-cues` (16 tests, 15 mutations red) built and tested, workspace bumped to 0.1.2 + CHANGELOG, PM allocates.
Open the ONE PR for 2026-11-20 only at/after 2026-11-20T00:00Z (`gh pr create`), then [READY]. Both -value and -cues go in that ONE PR (workspace 0.1.2; PM allocates).

## Next
English language packs under `packs/` with real checksums (self-authored, MIT OR Apache-2.0) so the crates ship usable data. Real packs for number-words, units, calendar, cues (English first, a second language and Rust docs to show the engine is language-free). A phrase in a pack cannot start with `#` (comment rule): note in RULES.

## Holds / WAITs
WAIT: open the day's PR | check: git ls-remote --heads origin value-day4 | wake: 2026-11-20T00:05:00Z
