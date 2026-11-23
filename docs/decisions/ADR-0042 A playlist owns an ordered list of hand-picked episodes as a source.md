---
type: decision
status: accepted
feature:
  - "[[playlists]]"
  - "[[playlists-rules]]"
  - "[[playlists-playback]]"
  - "[[playlists-editor]]"
related:
  - "[[ADR-0027 A playlist is a query over sources]]"
  - "[[US-0202 Add episodes to playlists]]"
  - "[[US-0204 Episodes can be reordered in playlists]]"
  - "[[US-0242 Order a playlist by hand with top and end pools]]"
  - "[[US-0243 See when a filter hides a hand-picked episode]]"
---
# A playlist owns an ordered list of hand-picked episodes as a source

## Context

An earlier draft of this decision expressed one-off episodes and pinning as rules. That would make the editor fill up with one rule per episode, make top and end pins interact unpredictably with the other sorts, and make drag-to-reorder within the pinned episodes awkward. [[US-0242 Order a playlist by hand with top and end pools]] wants a top group, one-offs and an end group, each with hand ordering.

## Decision

A playlist can own a **hand-picked list**: an ordered list of single episodes. It is a source in the query ([[ADR-0027 A playlist is a query over sources]]), but it exists only inside its playlist. It is not selectable as a source by other playlists, and it does not appear in the library or the Feed Selector. The editor shows it as a "Hand-picked" section inside the playlist.

- Each member has a position group: **top**, **one-off** (the default for an episode added by hand) or **end**. Order within the top and end groups is set by hand.
- **Filters apply to hand-picked episodes as normal.** A filter can remove one. The user needs to be able to see that it has happened ([[US-0243 See when a filter hides a hand-picked episode]]).
- The sorts order the rest of the playlist. Top members are then placed before everything else, and end members after everything else, so the pins are not reordered by the sorts. One-offs are ordered by the sorts like any other episode.
- Dragging and reordering are not MVP. Hand-picked lists as a whole are not MVP, as [[playlists-playback]] already says.

## Consequences

- The "Move to top / Move to bottom" menu actions set a member's group, adding it to the hand-picked list if needed.
- `DATA_MODEL` gains a hand-picked source: a playlist-owned ordered list of episode ids, each with a group.
- Still pure query on the whole: the hand-picked list is one more `FROM`, with top and end as fixed positions after the sorts.
- Open for the build: what happens to a member whose episode is deleted (for example on unsubscribe, see [[subscription-unsubscribe]]), and the exact wording and placement of the hidden-by-filter indicator.
