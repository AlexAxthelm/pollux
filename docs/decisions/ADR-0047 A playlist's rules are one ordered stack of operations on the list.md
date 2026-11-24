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

[[ADR-0027 A playlist is a query over sources]] describes a playlist as sources, then filters, then sorts. [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]] places top and end pins after the sorts. The user's current picture is different: filters, sorts, deduplication and the send-to-top and send-to-end actions are all operations on the list, applied in the order they are stacked, and they can be interleaved. This matters for deduplication, because "first" means the first occurrence at the point in the stack where Deduplicate sits, and that depends on whether a pin operation comes before or after it. The user will supply a draft of the editor interface before this is settled.

## Decision (draft, not settled)

The editor shows one ordered stack of operations applied to the combined list of sources:

- filters (each with include/exclude where it makes sense, [[ADR-0043 Every filter can include or exclude]]),
- sorts,
- **Deduplicate**, a filter with no criteria that keeps the first occurrence of each episode at that point,
- send to top and send to end for hand-picked episodes.

Each operation acts on the list as it is at its place in the stack. New playlists start with Deduplicate enabled at the top. It can be moved or turned off.

## Consequences

- If accepted, this changes ADR-0027's "filters, then sorts" and ADR-0042's "top and end are placed after the sorts". Both are flagged where they appear.
- Deduplicate has no include/exclude toggle, so [[ADR-0043 Every filter can include or exclude]] needs a note that it is on or off instead.
- #needs-decision **Are pins rows in the stack?** A pin could be one stack entry that places the whole hand-picked list's top and end groups, or each pin could be its own row, which is close to the rules approach that [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]] moved away from. The editor draft should settle it.
- The editor note ([[playlists-editor]]) separates Filters and Sort sections today and would change with this.
