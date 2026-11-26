---
type: decision
status: accepted
feature:
  - "[[playlists-playback]]"
related:
  - "[[ADR-0046 Queued episodes play before the active playlist resumes]]"
  - "[[ADR-0044 An episode's settings come from the context it is played from]]"
  - "[[ADR-0007 Playback source is separate from the episode's feed]]"
  - "[[ADR-0006 No auto-advance until playlists exist]]"
---
# The active playlist is set by where playback started

## Context

An episode can belong to many playlists, so "which playlist is this playing from" can't be derived from the episode. (Written in the playlists spec, April 2026; not yet built.)

## Decision

There is one active playlist at a time. Activating a new playlist evicts the previous one.

The active context is determined by **where in the UI the user initiated playback**, not by which playlists happen to contain that episode:

- Playing from a playlist view makes that playlist active.
- Playing from a subscription view makes that subscription (the implicit playlist) active.
- Playing from the library makes the **All subscriptions** convenience playlist active ([[playlists-convenience]]); the library is that playlist, not a separate implicit one.

Pressing play on an episode within a playlist view sets that playlist as active and begins playback at that episode. Pressing play on a playlist from the library resumes from the current position if it is already active, or begins from the first episode if not.

The active playlist persists through fully closing the app and reappears on next open. It becomes inactive only when it reaches its end.

## Consequences

- Each episode inherits its settings through the chain from the active playlist, but what plays next is whatever is next in the active playlist. See [[ADR-0044 An episode's settings come from the context it is played from]].
- Queueing an episode so it plays without changing the playlist ([[US-0152 Queue]]) is answered by a separate queue that plays before the active playlist resumes; see [[ADR-0046 Queued episodes play before the active playlist resumes]].
