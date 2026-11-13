---
type: decision
status: accepted
feature:
  - "[[player]]"
related:
  - "[[ADR-0012 The play context is restored on launch with capped save retries]]"
---
# Playback source is separate from the episode's feed

## Context

The player shows a "From:" row naming what playback was started from, and the
lock screen shows the episode's show as the artist. These can differ once
playlists exist.

## Decision

Playback remembers what it was started from as an `EpisodeSource` (a
subscription today; see `DATA_MODEL.md`), saved with the play context and
restored with it. The "From:" row names it and tapping it navigates back to it.
It is separate from the episode's own feed, which is what the lock screen shows
as the artist.

Playback started from a feed's episode list uses that feed; once playlists exist
the view that starts playback will say which source it is.

## Consequences

- Tapping "From: …" dismisses the full-player cover and shows the source (see
  "Presentation" in [[player]]).
