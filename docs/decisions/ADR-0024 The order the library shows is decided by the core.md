---
type: decision
status: accepted
feature:
  - "[[library-ordering]]"
related:
  - "[[US-0078 Locale ordering]]"
  - "[[US-0079 Ordering consistency]]"
---
# The order the library shows is decided by the core

## Context

Library ordering was defined in two places that could disagree. The storage query
orders subscriptions with `ORDER BY title COLLATE NOCASE` (`DatabaseManager.swift`),
which folds ASCII only, and the core sorts again with `title.to_lowercase()` when it
builds the view (`Pollux::view` in `shared/src/app.rs`).

## Decision

The order the library displays is decided by the core when it builds the view. The
storage query's order is not relied on.

## Consequences

- There is one place to change the ordering, not two that can disagree. Locale-aware
  collation (see [[library-ordering]]) only needs to land in the core.
- The storage query still has its own `ORDER BY title COLLATE NOCASE`. It is
  redundant, and still ASCII-only, but harmless because the core re-sorts. Removing
  it would make the decision harder to misread.
- The same holds for episode lists, which the core orders through `EpisodeSortOrder`.

## Notes for future

Since the ordering is determined by core, the DB ordering is currently redundant, but if we ever introduce pagination, then the queries will need to be kept in sync. Not an issue currently, since everything is small and local, so returning in one page is trivial.