---
type: decision
status: draft
feature:
  - "[[playlists-rules]]"
  - "[[playlists-editor]]"
related:
  - "[[ADR-0027 A playlist is a query over sources]]"
  - "[[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]"
  - "[[ADR-0043 Every filter can include or exclude]]"
  - "[[US-0217 Playlist deduplication]]"
  - "[[US-0218 Playlist allows duplicates]]"
---
# A playlist's rules are one ordered stack of operations on the list

## Context

[[ADR-0027 A playlist is a query over sources]] describes a playlist as sources, then filters, then sorts. The user's editor sketch shows one stack in which Deduplicate, filters and sorts can be dragged past each other, with a **Manual ordering** sort that appears when episodes are pinned ([[playlists-editor]]). This matters for Deduplicate, because "first" means the first occurrence in the order the list has at that point in the stack.

## Decision (draft, not settled)

A playlist has one ordered stack of rules, applied to the combined list of its sources:

- **Filters**, with include or exclude ([[ADR-0043 Every filter can include or exclude]]).
- **Deduplicate**, a filter with no criteria that keeps the first occurrence of each episode. New playlists start with it on, at the top. It can be moved or turned off.
- **Sorts**, including **Manual ordering** ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]), which appears when episodes are pinned and starts at the top.

Sorts are keys: the first sort in the stack is the primary key, the next breaks its ties, and so on, with source order as the last tiebreak. At any point in the stack, the list is ordered by the sorts above that point. Filters and Deduplicate then act on the list in that order.

Consequences of that: with Manual ordering at the top, pinned episodes come first or last, and Deduplicate below it keeps the pinned copy of an episode that also comes from another source. Moved below Deduplicate, it does not.

## Consequences

- If accepted, this changes ADR-0027's "filters, then sorts". It is flagged there.
- Deduplicate has no include/exclude toggle: it is on or off, not an exception to [[ADR-0043 Every filter can include or exclude]].
- #needs-decision **How do several sorts combine?** The text above says the first sort is the primary key. The other reading is that stacked rules are applied one after another, so a later sort is the primary key and an earlier one only breaks ties. That would put Manual ordering at the top of the stack in the weakest position. The first reading matches the old editor spec and the sketch; the user hasn't confirmed it.
- Which filters depend on the order of the list (Deduplicate, and later "limit total results") is where position in the stack matters. Most filters, such as "most recent N", use a field of the episode and not its position.
