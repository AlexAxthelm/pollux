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

**Each rule is applied to the list as if all the rules above it had already been evaluated.** A sort orders the list as it stands, so with several sorts the last one applied is the primary order and the ones above it only break ties. A filter such as "first N episodes" or Deduplicate acts on the list in the order it has at that point. The playlist does not have to be evaluated literally in that way, but the result must be the same as if it were.

Example: `sort: newest first`, `filter: first N episodes`, `sort: oldest first` gives the N most recent episodes, shown oldest first.

Deduplicate keeps the first occurrence in the order the list has at its place in the stack. If Manual ordering is applied above it, the pinned copy of an episode that also comes from another source is the one kept.

## Consequences

- If accepted, this changes ADR-0027's "filters, then sorts". It is flagged there.
- Deduplicate has no include/exclude toggle: it is on or off, not an exception to [[ADR-0043 Every filter can include or exclude]].
- #needs-decision **Where does Manual ordering start?** The decision is that it starts at the top of the stack. Because each rule is applied to the result of the ones above it, a sort lower down re-orders the whole list, pins included. With the sketch's rules (`sort: oldest first` below Manual ordering), a pinned episode would be moved by that sort and the pin would not hold. The pins only stay at the front and back if Manual ordering is applied after every other sort, that is at the bottom of the stack. Either it starts at the bottom, or it starts at the top and only breaks ties among episodes the later sorts treat as equal. Not decided.
- Which filters depend on the order of the list (Deduplicate, and later "limit total results") is where position in the stack matters. Most filters, such as "most recent N", use a field of the episode and not its position.
