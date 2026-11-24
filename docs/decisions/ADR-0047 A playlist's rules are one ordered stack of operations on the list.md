---
type: decision
status: accepted
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

[[ADR-0027 A playlist is a query over sources]] describes a playlist as sources, then filters, then sorts. One stack in which Deduplicate, filters and sorts can be dragged past each other, with a **Manual ordering** sort that appears, at the bottom (by default), when episodes are pinned ([[playlists-editor]]). This matters for Deduplicate, because "first" means the first occurrence in the order the list has at that point in the stack.

## Decision

A playlist has one ordered stack of rules, applied to the combined list of its sources:

- **Filters**, with include or exclude ([[ADR-0043 Every filter can include or exclude]]).
- **Deduplicate**, a filter with no criteria that keeps the first occurrence of each episode. New playlists start with it on, at the top. It can be moved or turned off.
- **Sorts**, including **Manual ordering** ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]), which appears when episodes are pinned and starts at the bottom of the stack.

**Each rule is applied to the list as if all the rules above it had already been evaluated.** A sort orders the list as it stands, so with several sorts the last one applied is the primary order and the ones above it only break ties. A filter such as "first N episodes" or Deduplicate acts on the list in the order it has at that point. The playlist does not have to be evaluated literally in that way, but the result must be the same as if it were.

Example: `sort: newest first`, `filter: first N episodes`, `sort: oldest first` gives the N most recent episodes, shown oldest first.

**With no sorts, the order is the source order**, each source in its own natural order. Example: all of playlist A, then show B, then playlist C.

Deduplicate keeps the first occurrence in the order the list has at its place in the stack. At the top of the stack that is source order. A pin belongs to the episode, so it applies to the copy that survives, which may come from a different source than the one it was pinned from; that affects the route its settings follow ([[ADR-0044 An episode's settings come from the context it is played from]]).

## Consequences

- This changes ADR-0027's "filters, then sorts", which has been updated to match.
- Deduplicate has no include/exclude toggle: it is on or off, not an exception to [[ADR-0043 Every filter can include or exclude]].
- **Manual ordering starts at the bottom of the stack.** Because each rule is applied to the result above it, a sort lower down would re-order the pins, so the row starts last, where it is applied after every other sort and the pins hold. The user can move it up, and then later sorts apply to the pinned result like any other.
- Which filters depend on the order of the list (Deduplicate, and later "limit total results") is where position in the stack matters. Most filters, such as "most recent N", use a field of the episode and not its position.
- A source's **natural order** is: for a playlist, whatever its own rules resolve to; for a show, newest first by default; for the hand-picked list, the order the episodes were added, oldest first, so the most recent addition is last. The hand-picked list therefore stores when each episode was added.
