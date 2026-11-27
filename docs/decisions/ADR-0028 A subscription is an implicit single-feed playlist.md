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

- A subscription's implicit playlist has no filters, so advancing from it goes down its list, played episodes included. Better behavior for it (and for All subscriptions), probably only when it is the active context, is to follow up on later ([[playlists-convenience]]).
- A subscription's implicit playlist has **one built-in rule, its Sort** (Newest first, Oldest first, Title A–Z, as on the show page). The visible sort is the playlist's order, not a display preference. When the show is the active context, advancing follows that order, so what the listener sees is what plays, and a change to the sort while it plays takes effect at the next advance. When the show is a **source of another playlist**, its sort does not travel: it contributes its natural order, newest first, so a display choice on the show page cannot reorder someone's playlists ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]).

## Consequences

- This is already reflected in the code: `EpisodeSource` (`shared/src/domain/episode_source.rs`) has only the subscription variant today, and a `Playlist { id }` variant is added when playlists exist.
- Playback started from a subscription's episode list activates that subscription; see [[ADR-0029 The active playlist is set by where playback started]].
