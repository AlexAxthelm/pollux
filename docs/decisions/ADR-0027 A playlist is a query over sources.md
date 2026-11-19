---
type: decision
status: accepted
feature:
  - "[[playlists]]"
  - "[[playlists-rules]]"
related:
  - "[[US-0201 Playlists can be filtered]]"
---
# A playlist is a query over sources

## Context

Playlists are the primary way users organize and consume episodes. The model should not have to change each time a new kind of filter or sort is wanted. (Written in the playlists spec, April 2026; not yet built, Phase 4 in `ROADMAP.md`.)

## Decision

A playlist is fundamentally a **query** over a set of sources, with filters and sorts applied:

```
(sources) → [filters] → [sorts] → ordered episode list
```

- **Sources** are one or more feed subscriptions, one or more other playlists (composite / meta-playlists), or a mix of both.
- **Filters** narrow which episodes are included; **sorts** order the result. Rules are applied in sequence, and order matters.
- A playlist with no rules includes all episodes from its sources. An empty source set with a rule applied acts on all episodes from all subscribed feeds; a rule applied to a populated source set acts only on that set.
- The rule system is intentionally analogous to a SQL SELECT: sources = FROM, filters = WHERE, sorts = ORDER BY.

## Consequences

- The model is intentionally complete from the start. MVP scope is limited by which filter and sort *operations* are implemented, not by the model itself.
- Each new rule type can be added incrementally without changing the underlying data model.
- Playlists that include playlists make cycles possible; see [[ADR-0030 Playlist resolution is capped at 1,000 episodes instead of detecting cycles]].
