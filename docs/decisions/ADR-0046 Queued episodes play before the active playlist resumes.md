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

There is a separate, ephemeral **queue** ("up next") alongside the active playlist.

- Queued episodes play, in order, before the active playlist resumes.
- Queueing never changes the active playlist, its source, or its position.
- When the queue empties, playback continues in the active playlist where it left off.
- The queue is not a playlist: it has no rules, and it is not shown in the library.
- Each queue entry carries the **context** it was queued from (the route of playlists down to the episode, as in [[ADR-0044 An episode's settings come from the context it is played from]]). When it plays, its settings resolve along that route, so an episode queued from playlist A takes A's values. An episode queued from a subscription or the library resolves as subscription → global.
- The queue row shows a `from:` line naming that context, like the player's "From:" row.

## Consequences

- ADR-0029 is unchanged. The queue sits beside it rather than replacing any part of it.
- The saved play context needs a queue as well as the active source: an ordered list of entries, each an episode id plus its context route.
- Open for the build: whether the queue survives a cold start (the active playlist does), and how the user adds and clears items. Settle these when the queue is specified.
