---
type: decision
status: accepted
feature:
  - "[[player-streaming]]"
  - "[[downloads-queue]]"
related:
  - "[[ADR-0003 Streaming starts at once and swaps to the download]]"
  - "[[US-0159 Remember cancelled downloads]]"
---
# Playback never downloads against the listener's wishes

## Context

Playback queues downloads on its own ([[ADR-0003 Streaming starts at once and swaps to the download]]).
A listener who has cancelled or deleted an episode's download has said what they
want, and playback shouldn't quietly undo it.

## Decision

Once the listener cancels or deletes an episode's download, starting or resuming
it, or replacing a file the engine lost, streams without fetching it again. Only
asking for the download (the Download action) lifts that.

## Consequences

- The memory (`declined_downloads`) is kept for the session only. After a
  relaunch a declined download may start again. Making it durable needs a column
  or table; see [[US-0159 Remember cancelled downloads]] and "Declined downloads
  and the bad-file memory last for the session" in [[player-streaming]].
