---
type: decision
status: accepted
feature:
  - "[[playlists-playback]]"
  - "[[player]]"
related:
  - "[[US-0152 Queue]]"
  - "[[ADR-0029 The active playlist is set by where playback started]]"
---
# Queued episodes play before the active playlist resumes

## Context

[[US-0152 Queue]] asks to queue an episode so it plays without changing the playlist being listened to. [[ADR-0029 The active playlist is set by where playback started]] has one active playlist and no queue, and activating a playlist evicts the previous one.

## Decision

There is a separate **queue** ("up next") alongside the active playlist. It holds only the episodes the user has queued and empties as they play.

- Queued episodes play, in order, before the active playlist resumes.
- Queueing never changes the active playlist, its source, or its position.
- When the queue empties, playback continues in the active playlist where it left off. "Where it left off" is kept as an **anchor**: the playlist episode that was playing (or had just finished) when the queue took over, with its route. When the queue empties, the playlist is resolved with that anchor still counted as in place, as at an ordinary advance ([[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]), and "next" is whatever follows it, so edits and new episodes made while the queue played are reflected. If the anchor is no longer in the list, or there was none (the queue started with nothing playing, or the active playlist was replaced), playback ends when the queue empties.
- The queue is not a playlist: it has no rules, and it is not shown in the library.
- Each queue entry carries the **context** it was queued from (the route of playlists down to the episode, as in [[ADR-0044 An episode's settings come from the context it is played from]]). When it plays, its settings resolve along that route, so an episode queued from playlist A takes A's values. An episode queued from a subscription or the library resolves as subscription → global.
- The queue row shows a `from:` line naming that context, like the player's "From:" row.

## Consequences

- ADR-0029 is unchanged. The queue sits beside it rather than replacing any part of it.
- A queue entry's route is fixed when it is queued and may be old by the time it plays. If a playlist on it has since been deleted or dropped as a source, that level is skipped and the rest of the route is used, so a queued episode keeps the settings of where the user picked it. The route itself is not recomputed at play time, but the setting values are resolved along it when the entry starts, using current settings.
- The saved play context needs a queue as well as the active source: an ordered list of entries, each an episode id plus its context route, and the anchor (an episode id plus its route). When a queued entry starts it leaves the pending queue and becomes the active episode, with its own route, so a restart reopens it and does not repeat or omit it.
- The queue persists across restarts, like the active playlist. After a restart with episodes in the queue they are still there, and so is the anchor. On a first launch it is empty.
- **Adding:** **Add to queue** is in an episode's "more" menu, not a top-level action like Play, and adds the episode to the **end** of the queue. Adding never interrupts: the current episode plays to its end, then whatever is at the top of the queue plays. A way to add to the top of the queue is a later story ([[US-0260 Add an episode to the top of the queue]]); it is not called "Play next", which is the playlist setting ([[ADR-0044 An episode's settings come from the context it is played from]]).
- **The player stays active through the queue,** across restarts too, because the next queued episode loads as the previous one finishes. The Up next sheet is reached from an active player. Whether and how it can be reached when the player is inactive with items queued is not settled; the only inactive case would be a sleep timer, which is not built.
- **The Up next sheet** (details later) is opened from the player. It lists the queue entries with their `from:` lines, then a divider ("Then from" the active playlist) and the next few episodes of the re-resolved playlist. Those are a read-only preview and can change.
- **Editing:** reordering and removing are available behind an **Edit** button on the sheet, with no handles shown by default.
- **Clearing** asks for confirmation, with a press-and-hold. An app setting can turn the confirmation off.
- **A queued episode that is also in the active playlist plays twice.** If episode 2 of the playlist is queued while episode 1 plays, it plays from the queue, and the playlist then resumes after episode 1, so episode 2 plays again unless a filter such as "not played" now hides it. That is the expected behavior for a playlist that does not hide played episodes. To revisit after using it: skipping episodes already played from the queue since the playlist paused, which would need that list persisted.
- **To settle when the queue is built (queue drain and "From the top").** The queue always resumes by resolving around the anchor and taking what follows it. If the active playlist is set to "From the top", an episode that arrived above the anchor while the queue played would be skipped by this rule and picked by the other. Decide whether queue drain follows the playlist's next-episode setting, and whether changing that setting while queued audio plays takes effect.
- **To settle when the queue is built (unsubscribe).** Unsubscribing deletes the show's episodes ([[subscription-unsubscribe]]). Say what happens to pending queue entries for it, and to the anchor when the show that supplied it is removed, without clearing an unrelated queued episode that is playing. Skipping invalid pending entries, and treating a missing anchor as "no anchor" (playback ends when the queue empties), would fit the rules above.
