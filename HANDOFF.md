# HANDOFF: semanticant

You own `C:\Projects\semanticant` (github.com/ThinkersJournal/semanticant, public). Read `C:\Projects\CLAUDE.md`, `docs/SPEC-STEP1.md`.
Sonnet only; never Opus. Primary stays on `main`; branches in worktrees (`C:\Projects\semanticant-wt-<topic>`).

## State (2026-11-18)
`semanticant-core` 0.1.0 published. PR #3 (`text-day2`, worktree `semanticant-wt-text`) is DRAFT by PM HOLD (third PR of the UTC day): it now holds `-text`, `-pack`, deny job, version 0.1.1 + CHANGELOG.
At 2026-11-19T00:00Z or later: `gh pr ready 3`, retitle to cover text+pack, send [READY]. ONE PR per day: fold the day's work into the open one, never a second.
Deviations from SPEC 3.2: `fold(text, rules)` has no lang arg; `sentences` returns `Vec<Verdict<Span>>`. Symlink test in -pack is unix-only (runs on CI ubuntu).

## Next
After #3 is ready/merged: `-lexicon` (day 11-19 PR), then `-value`, `-cues`. PM adds job `deny` to protection after its first green run on main. After merge: delete branch + worktree.

## Holds / WAITs
WAIT: open #3 ready at UTC day change | check: gh pr view 3 --json state,isDraft | wake: 2026-11-19T00:05:00Z
