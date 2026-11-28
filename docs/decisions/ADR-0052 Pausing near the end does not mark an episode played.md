---
type: decision
status: accepted
feature:
  - "[[player-progress]]"
  - "[[playlists-playback]]"
related:
  - "[[ADR-0002 Played is decided when the listener leaves an episode]]"
  - "[[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]"
---
# Pausing near the end does not mark an episode played

## Context

[[ADR-0002 Played is decided when the listener leaves an episode]] says an episode becomes played when the engine reaches the end, when the listener pauses, or when they start another episode, if it is within the played tolerance of the end. With playlists, a pause near the end would mark the episode played, clear the saved context and make the playlist inactive, so resuming would not carry on in the playlist.

## Decision

Pausing near the end does **not** mark the episode played and does not clear the context. An episode is marked played, and the context changes, only when the context itself changes: the episode actually ends, or the user manually starts something else. (The played tolerance still applies at those two moments.)

## Consequences

- This changes the "when they pause" part of ADR-0002. ADR-0002 notes that it is partly superseded by this ADR, and [[player-progress]] says so. The player does not do this yet.
- A paused episode near its end stays in progress, keeps its position and its playlist context, and play resumes it. It survives a cold start in the same way.
- System pauses are unchanged. Playlist advance still happens only when the episode ends ([[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]).
