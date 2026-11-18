# HANDOFF: semanticant

You own `C:\Projects\semanticant` (github.com/ThinkersJournal/semanticant, public). Read `C:\Projects\CLAUDE.md`, `docs/SPEC-STEP1.md`.
Sonnet only; never Opus. Primary stays on `main`; branches in worktrees (`C:\Projects\semanticant-wt-<topic>`).

## State (2026-11-18)
Spec merged (PR #1). PM rulings: pack format text-first; `-value`/`-cues` split; MSRV 1.88; core hosts Reason/Verdict/Provenance/pack Manifest/Span/LangTag, stdlib only, no identifiers.
Day 1 = `semanticant-core` ONLY, PR `core-day1` (worktree `semanticant-wt-core`): 11 tests, 7 sabotage mutations all red. Version 0.1.0 proposed; PM allocates at [READY] and publishes (siblings need it on crates.io).

## Next
Day 2 `semanticant-text` (after core merges/publishes), then pack loader, lexicon, value, cues (SPEC-STEP1 section 6). One PR per day.
Delete merged branch + worktree per CLAUDE.md section 3.

## Holds / WAITs
WAIT: core PR merge | check: gh pr view 2 --json state,mergedAt | wake: 2026-11-19T12:00:00Z
