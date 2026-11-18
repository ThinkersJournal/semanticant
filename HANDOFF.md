# HANDOFF: semanticant

You own `C:\Projects\semanticant` (github.com/ThinkersJournal/semanticant, public). Read `C:\Projects\CLAUDE.md`, `docs/SPEC-STEP1.md`.
Sonnet only; never Opus. Primary stays on `main`; branches in worktrees (`C:\Projects\semanticant-wt-<topic>`).

## State (2026-11-18)
`semanticant-core` 0.1.0 merged and PUBLISHED (siblings depend on it). Day 2 = `semanticant-text` + deny.toml + CI job `deny`, PR `text-day2` (worktree `semanticant-wt-text`): 12 tests, 11 mutations red.
Version: workspace 0.1.0 carried; PM allocates the next at gate. Branch protection requires msrv, test ubuntu, test windows: PM adds `deny`.
Deviations from SPEC-STEP1 3.2 (told in PR): `fold(text, rules)` has no lang arg (tailoring is data in `FoldRules`); `sentences` returns `Vec<Verdict<Span>>`.

## Next
Day 3 `semanticant-pack` (loader: pack.toml parse + sha256 verify, text first) then `-lexicon`, `-value`, `-cues`. One PR per day. After merge: delete branch + worktree.

## Holds / WAITs
WAIT: text PR merge | check: gh pr view 3 --json state,mergedAt | wake: 2026-11-19T12:00:00Z
