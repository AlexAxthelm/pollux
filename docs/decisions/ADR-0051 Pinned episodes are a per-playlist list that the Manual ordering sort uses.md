---
type: decision
status: accepted
feature:
  - "[[playlists]]"
  - "[[playlists-rules]]"
  - "[[playlists-editor]]"
related:
  - "[[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]"
  - "[[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]"
  - "[[US-0242 Order a playlist by hand with top and end pools]]"
---
# Pinned episodes are a per-playlist list that the Manual ordering sort uses

## Context

The hand-picked list ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]) decides which extra episodes are in a playlist. Moving an episode to the top or the end is a different thing: it decides the order of episodes that are already in the list. When the pin was stored on a hand-picked member, pinning an episode that comes from a show added a second copy of it to the hand-picked list. With Deduplicate on, the show's copy could survive and the pin was lost with the dropped copy; with it off, the episode appeared twice.

## Decision

Hand-picked is a source and pinning is a sort, and they are kept apart.

- A playlist stores a list of **pins**. Each pin is one **occurrence** of an episode: the episode id and the source it came through, plus a group (**top** or **end**) and its place in that group, set by hand.
- "Move to top" and "Move to bottom" create or change a pin. The reset button on a pinned episode removes it. Neither adds anything to a source.
- The **Manual ordering** sort moves pinned episodes to the front or the back of the list, in their hand-set order, whichever kind of source it came from (a show, an included playlist or the hand-picked list). Only the pinned occurrence moves. If the same episode appears three times from different sources and the middle one is pinned to the top, the other two stay where they are. The row appears when the playlist has a pin, starts at the bottom of the stack, can be moved like any other sort, and can be removed once nothing is pinned ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]).
- A pin whose occurrence is not in the list because a filter or Deduplicate removes it has no visible effect and stays until it is removed, or until its source stops providing the episode.
- Pinning does not add an episode to the hand-picked list, and removing an episode from the hand-picked list does not unpin it.

## Consequences

- `DATA_MODEL` gains a pin entity: playlist, episode id, source, group and order.
- When a show is unsubscribed, its episodes are deleted and any pins for them are removed in the same transaction ([[subscription-unsubscribe]]).
- Pins do not count as members for retention or downloads. Hand-picked episodes do ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]).
- A pin is removed when its source no longer provides the episode: the source is removed from the playlist, or the episode leaves the source (for example its show is unsubscribed). This includes an included playlist that stops providing it: if a pinned episode comes through an "unplayed" playlist and is then played, that playlist no longer contains it, so the pin is removed. A pin that is only hidden, because a filter or Deduplicate drops that occurrence, stays and has no visible effect until the occurrence returns.
- **The Manual ordering row stays movable, with cues.** Moving it up is the only way to keep a pinned episode through a cut: a pin below First N episodes does nothing for an episode the cut already dropped. The cost is that a pin can look broken when a sort sits below the row. So the row shows a cue when a sort is below it ("Sorts below this will reorder pinned episodes") and another when First N episodes is below it (pins apply before the cut). To explore in practice: if the cues are not enough, switch to **fixed at the bottom**, accepting that a pin cannot survive a cut.
- To explore in practice: letting a pin fall back to the episode's first remaining occurrence when its own occurrence goes away, so that a change to Deduplicate or to the source order does not leave it without a match. For now it does not.
