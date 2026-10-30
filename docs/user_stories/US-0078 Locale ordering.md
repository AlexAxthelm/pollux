---
type: user-story
status: draft
feature:
  - "[[library]]"
  - "[[settings]]"
related: []
---
As a subscriber whose shows aren't all in English
I want my library sorted the way my language sorts
so that a show starting with "Ä" or "É" appears where I expect it rather than
after everything else

Notes from current state:
*  Current behavior is naive code-point ordering, not locale-aware collation.
  `Pollux::view` sorts with `sort_by_cached_key(|s| s.title.to_lowercase())`,
  which folds `Ä` to U+00E4 — past ASCII `z` at U+007A. Measured output for a
  mixed list is
  `["apple", "Banana", "eagle", "Zebra", "Ärger", "Ångström", "Émile"]`.
  That matches no real locale: German sorts `Ä` with `A`, Swedish places it after
  `Z` but ordered `Å < Ä < Ö`, and we emit `Ärger` before `Ångström`.
- The SQLite side had the same blind spot — `ORDER BY title COLLATE NOCASE` in
  `DatabaseManager.swift` folds ASCII only — so this was never correct on either
  side. Sorting now happens only in the core, which at least means one place to
  fix rather than two that can disagree.
- A real fix means locale-aware collation (e.g. the `icu_collator` crate) plus a
  decision about where the locale comes from. That is a dependency and
  architecture call, so per [[CONTRIBUTING]] it wants discussion
  before implementation rather than being folded into a feature PR.
- Deliberately **not** covered by a test. Asserting the current order would pin
  behavior no locale actually wants and would make a correct implementation later
  look like a regression.
- Related: [[library]] sets alphabetical as the default ordering
  for both Playlists and Shows, with "by last updated" and custom order as later
  additions — whatever collation lands here should apply to those too.
