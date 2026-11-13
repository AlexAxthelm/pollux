---
type: decision
status: draft
feature:
  - "[[library-ordering]]"
related:
  - "[[US-0078 Locale ordering]]"
  - "[[ADR-0024 The order the library shows is decided by the core]]"
---
# Library sorting uses locale-aware collation

## Context

A listener whose shows aren't all in English should see the library sorted the way
their language sorts, so a show starting with "Ä" or "É" appears where they expect
it rather than after everything else.

**Current behavior** is naive code-point ordering, not locale-aware collation. The
core sorts with `sort_by_cached_key(|s| s.title.to_lowercase())` in `Pollux::view`,
which folds `Ä` to U+00E4, past ASCII `z` at U+007A. For a mixed list the measured
output is:

```
["apple", "Banana", "eagle", "Zebra", "Ärger", "Ångström", "Émile"]
```

That matches no real locale. German sorts `Ä` with `A`. Swedish places it after `Z`,
but ordered `Å < Ä < Ö`. We emit `Ärger` before `Ångström`.

The SQLite side had the same blind spot (`ORDER BY title COLLATE NOCASE` in
`DatabaseManager.swift` folds ASCII only), so this was never correct on either side.
Because the core decides the order shown
([[ADR-0024 The order the library shows is decided by the core]]), there is one
place to fix, not two that can disagree.

## Decision

**Not yet decided.** A real fix means locale-aware collation in the core (for
example the `icu_collator` crate), plus a decision about where the locale comes
from.

Open:

- Which collation implementation, and what dependency does it add?
- Where does the locale come from (the device language, a user setting, something
  else)?

That is a dependency and architecture call, so per [[CONTRIBUTING]] it wants
discussion before implementation, not to be folded into a feature PR.

## Consequences

- Whatever collation lands should also apply to the later ordering options in
  [[library-ordering]] (by last updated, custom order).
- The current behavior is deliberately **not** covered by a test. Asserting the
  current order would pin behavior no locale actually wants, and would make a correct
  implementation later look like a regression.
