---
type: decision
status: accepted
feature:
  - "[[playlists]]"
  - "[[playlists-rules]]"
related:
  - "[[ADR-0050 A playlist with no sources is empty]]"
  - "[[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]"
  - "[[US-0201 Playlists can be filtered]]"
---
# A playlist is a query over sources

## Context

Playlists are the primary way users organize and consume episodes. The model should not have to change each time a new kind of filter or sort is wanted. (Written in the playlists spec, April 2026; not yet built, Phase 4 in `ROADMAP.md`.)

## Decision

A playlist is fundamentally a **query** over a set of sources, with filters and sorts applied:

```
(sources) → [rules: filters, sorts, deduplicate, in one ordered stack] → ordered episode list
```

- **Sources** are one or more feed subscriptions, one or more other playlists (composite / meta-playlists), or a mix of both. Two more kinds exist: the **All shows** source, which follows the library ([[ADR-0050 A playlist with no sources is empty]]), and the playlist's own **hand-picked list** ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]; not MVP). The UI may expose affordances to the user to simplify complex operations/combinations.
- **Filters** narrow which episodes are included; **sorts** order the result. Rules are applied in sequence, each to the result of the ones above it, and order matters. Filters and sorts can be interleaved ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]).
- A playlist with no rules includes all episodes from its sources. A playlist with no sources is an empty list ([[ADR-0050 A playlist with no sources is empty]]; this replaced the earlier rule that an empty source set acts on all subscribed feeds).
- The rule system is intentionally analogous to a SQL SELECT: sources = FROM, filters = WHERE, sorts = ORDER BY, though rules can be interleaved in a stack.

## Consequences

- The model is intentionally complete from the start. MVP scope is limited by which filter and sort *operations* are implemented, not by the model itself.
- Each new rule type can be added incrementally without changing the underlying data model.
- Playlists that include playlists make cycles possible; see [[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]] and [[ADR-0053 Resolution is bounded by the work it does, and playlists have no episode limit]].
- The rules form one ordered stack and filters and sorts can be interleaved, as decided in [[ADR-0047 A playlist's rules are one ordered stack of operations on the list]].
