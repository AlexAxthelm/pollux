---
type: decision
status: accepted
feature:
  - "[[playlists]]"
related:
  - "[[ADR-0007 Playback source is separate from the episode's feed]]"
---
# A subscription is an implicit single-feed playlist

## Context

Playback logic should only have to understand playlists, not whether the source is a subscription or a user-created playlist.

## Decision

A feed subscription behaves as an implicit single-feed playlist. It responds to the same play and queue actions as any other playlist. Playing from the library plays the All subscriptions convenience playlist ([[playlists-convenience]]).

## Consequences

- This is already reflected in the code: `EpisodeSource` (`shared/src/domain/episode_source.rs`) has only the subscription variant today, and a `Playlist { id }` variant is added when playlists exist.
- Playback started from a subscription's episode list activates that subscription; see [[ADR-0029 The active playlist is set by where playback started]].
