---
type: decision
status: accepted
feature:
  - "[[playlists-playback]]"
related:
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
- Playing from the library makes the library (the implicit all-subscriptions playlist) active.

Pressing play on an episode within a playlist view sets that playlist as active and begins playback at that episode. Pressing play on a playlist from the library resumes from the current position if it is already active, or begins from the first episode if not.

The active playlist persists through fully closing the app and reappears on next open. It becomes inactive only when it reaches its end.

## Consequences

- Queueing an episode so it plays without changing the playlist ([[US-0152 Queue]]) has no counterpart in this model; see the open question in [[playlists-playback]].
