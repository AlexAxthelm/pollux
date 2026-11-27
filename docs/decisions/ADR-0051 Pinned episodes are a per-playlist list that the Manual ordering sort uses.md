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

The hand-picked list ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]) decides which extra episodes are in a playlist. Moving an episode to the top or the end is a different thing: it decides the order of episodes that are already in the list. When the pin was stored on a hand-picked member, pinning an episode that comes from a show added a second copy of it to the hand-picked list. With Deduplicate on, the show's copy could survive and the pin was lost with the dropped copy; with it off, the episode appeared twice. Pins are now their own list, keyed by the episode.

## Decision

Hand-picked is a source and pinning is a sort, and they are kept apart.

- A playlist stores a list of **pins**. Each pin is an **episode** in that playlist, with a group (**top** or **end**) and its place in that group, set by hand. A pin is keyed by the episode, not by the source it came through.
- "Move to top" and "Move to bottom" create or change a pin. The reset button on a pinned episode removes it. Neither adds anything to a source. Pins are changed in the editor and saved with **Save**, like everything else on that screen.
- The **Manual ordering** sort moves pinned episodes to the front or the back of the list, in their hand-set order, whichever kind of source they came from (a show, an included playlist or the hand-picked list). If the same episode appears more than once (Deduplicate is off), the pin moves its first occurrence in the list at that point, and the other copies stay where they are. The row appears when the playlist has a pin, starts at the bottom of the stack, can be moved like any other sort, and can be removed once nothing is pinned ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]).
- A pin whose episode is in some source but is hidden by a filter or Deduplicate has no visible effect and stays.
- Pinning does not add an episode to the hand-picked list. Removing an episode from the hand-picked list unpins it only if no other source provides the episode (see Consequences).

## Consequences

- `DATA_MODEL` gains a pin entity: playlist, episode id, group and order.
- When a show is unsubscribed, its episodes are deleted and any pins for them are removed in the same transaction ([[subscription-unsubscribe]]).
- Pins do not count as members for retention or downloads. Hand-picked episodes do ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]).
- **A pin is removed when no source provides the episode any more:** the source that supplied it is removed from the playlist, the episode leaves every source (for example its show is unsubscribed), or an included playlist stops providing it. A pin that is only hidden, because a filter or Deduplicate drops the episode in this playlist, stays and comes back into effect when the episode returns.
- **When, and for which playlist:** resolution itself changes nothing, and the editor's preview never deletes a pin. Pins are cleaned up only for the playlist that is resolved **as the root**: the one that was opened, saved or is playing. Its result does not depend on a path, so it is the same however the playlist is reached. The cleanup removes that playlist's own pins that no source provides, when the playlist is saved and in a separate step after a resolution for playback or for viewing it. Resolving a playlist never cleans the pins of the playlists it includes, because an included playlist can resolve differently inside a loop ([[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]). Example: A includes B and C, and B includes A and D. An episode from C pinned in B stays pinned when A is viewed, even though B provides it only when opened on its own; inside A that pin is dormant, not deleted.
- **Accepted consequence of keying by episode:** if an episode is provided by two sources, a pin made when it came through one of them stays when the other stops providing it. Example: sources are Show A and a "not played" playlist U that includes A; E is pinned; E is played, U drops it, but Show A still provides it, so E stays pinned at the top, played, until the user resets it. Keying the pin by source would remove it, at the price of the orphaned pins that reordering sources or removing and re-adding one would cause.
- **The Manual ordering row stays movable, with cues.** Moving it up is the only way to keep a pinned episode through a cut: a pin below First N episodes does nothing for an episode the cut already dropped. The cost is that a pin can look broken when a sort sits below the row. So the row shows a cue when a sort is below it ("Sorts below this will reorder pinned episodes") and another when First N episodes is below it (pins apply before the cut). To explore in practice: if the cues are not enough, switch to **fixed at the bottom**, accepting that a pin cannot survive a cut.
- A pinned episode that comes from several sources can have different resolved settings in each copy. A small warning for that is a later story ([[US-0253 Be told when a pinned episode plays with different settings by source]]).
