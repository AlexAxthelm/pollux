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

[[ADR-0027 A playlist is a query over sources]] describes a playlist as sources, then filters, then sorts. This ADR makes it one stack, in which Deduplicate, filters and sorts can be dragged past each other, with a **Manual ordering** sort for pins ([[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]). This matters for Deduplicate, because "first" means the first occurrence in the order the list has at that point in the stack.

## Decision

A playlist has one ordered stack of rules, applied to the combined list of its sources:

- **Filters**, with include or exclude ([[ADR-0043 Every filter can include or exclude]]).
- **Deduplicate**, a filter with no criteria that keeps the first occurrence of each episode. New playlists start with it on, at the top. It can be moved or turned off.
- **Sorts**, including **Manual ordering** ([[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]), which appears when episodes are pinned and starts at the bottom of the stack.

**Each rule is applied to the list as if all the rules above it had already been evaluated.** A sort orders the list as it stands, so with several sorts the last one applied is the primary order and the ones above it only break ties. A filter such as "first N episodes" or Deduplicate acts on the list in the order it has at that point. The playlist does not have to be evaluated literally in that way, but the result must be the same as if it were.

Example: `sort: newest first`, `filter: first N episodes`, `sort: oldest first` gives the N most recent episodes, shown oldest first.

**With no sorts, the order is the source order**, each source in its own natural order. Example: all of playlist A, then show B, then playlist C.

At the top of the stack, Deduplicate's "first" is source order. How a pin interacts with it is in [[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]].

## Consequences

- The last sort being the main order is the reverse of the usual "first sort is primary", so the MVP editor shows a result count under each rule and a line of helper text on this ([[playlists-editor]]). A fixed filters-then-sorts order was considered and rejected: it cannot express "the N most recent, shown oldest first" or the archive ordering ([[playlists-archive]]).
- This changes ADR-0027's "filters, then sorts", which has been updated to match.
- Filters only intersect: each one narrows what the rules above it left, so there is no OR between filters. To get "A or B", add both as sources. An OR between filters can be specified later.
- A playlist has at most one Deduplicate. It is idempotent, so a second one would only differ if a "First N episodes" filter sat between them. "+ Filter" does not offer it while one is in the stack; if it is removed and added again, it goes back at the top, as in a new playlist.
- Deduplicate has no include/exclude toggle: it is on or off, not an exception to [[ADR-0043 Every filter can include or exclude]].
- **Manual ordering starts at the bottom of the stack.** Because each rule is applied to the result above it, a sort lower down would re-order the pins, so the row starts last, where it is applied after every other sort and the pins hold. The user can move it up, and then later sorts apply to the pinned result like any other.
- Which filters depend on the order of the list (Deduplicate and "First N episodes") is where position in the stack matters. Most filters, such as "N per source" or playback status, use a field of the episode and not its position.
- A source's **natural order** is: for a playlist, whatever its own rules resolve to; for a show, newest first by default; for the hand-picked list, the order the episodes were added, oldest first, so the most recent addition is last; for the All shows source, all episodes of all subscribed shows merged by release date, newest first, like one big show. A show-by-show order comes from the "by source" sort, not from the source. The hand-picked list therefore stores when each episode was added.
