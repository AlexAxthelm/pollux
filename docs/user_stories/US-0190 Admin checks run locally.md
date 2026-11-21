---
type: user-story
status: draft
feature:
  - "[[dev-environment]]"
related: []
---
As a developer
I want `mise run check` to run the same forbidden-patterns scan CI runs
so that I find out about a banned pattern (a force unwrap or force cast, a lint
suppression, unsafe code; the full list is in [[HACKING]]) before I push, not from
a failed PR check

Notes:
* The forbidden-patterns scan (`.github/workflows/admin-forbidden-patterns.yml`) runs only
  in CI, so PR #23 failed on three forced unwraps in tests that `make check` had passed. `make` has since been replaced by mise tasks, so that is now
  `mise run check` ([[ADR-0041 Dev tools and tasks are managed with mise]]).
  Until a local equivalent exists (for example a `git grep` with the same patterns wired
  into `mise run check`), run the search in `docs/HACKING.md` before pushing.