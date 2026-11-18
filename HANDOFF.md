# HANDOFF: semanticant

You own `C:\Projects\semanticant` (github.com/ThinkersJournal/semanticant, public). Read `C:\Projects\CLAUDE.md`, `docs/SPEC-STEP1.md`.
Sonnet only; never Opus. Primary stays on `main`; branches in worktrees (`C:\Projects\semanticant-wt-<topic>`).

## State (2026-11-18)
`semanticant-core` 0.1.0 published. PR #3 (`text-day2`, worktree `semanticant-wt-text`) is DRAFT by PM HOLD (third PR of the UTC day): it now holds `-text`, `-pack`, `-lexicon`, deny job, version 0.1.1 + CHANGELOG. Tests: core 11, text 11, pack 11, lexicon 12.
At 2026-11-19T00:00Z or later: `gh pr ready 3`, retitle to cover text+pack, send [READY]. ONE PR per day: PM ruling: #3 is the DAY'S PR for 11-19 (ready at/after 00:00Z with what is finished); 11-20 work goes in a NEW PR opened 11-20, never into this branch after it merges.
Deviations from SPEC 3.2: `fold(text, rules)` has no lang arg; `sentences` returns `Vec<Verdict<Span>>`. Symlink test in -pack is unix-only (runs on CI ubuntu).

## Next
After #3: 11-20 PR = `-value`, then `-cues` (new branch off main). PM adds job `deny` to protection after its first green run on main. After merge: delete branch + worktree.

## Holds / WAITs
WAIT: open #3 ready at UTC day change | check: gh pr view 3 --json state,isDraft | wake: 2026-11-19T00:05:00Z
