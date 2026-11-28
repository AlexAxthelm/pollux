---
type: decision
status: accepted
feature:
  - "[[player-os-integration]]"
related:
  - "[[US-0137 System control]]"
---
# Next and previous track are disabled on the lock screen

## Context

Lock-screen and Control Center controls offer next/previous track. Podcast
convention is to use skip buttons there instead.

## Decision

Next/previous track commands are disabled. The skip commands (30s forward, 15s
back) stand in for them, per podcast convention.

## Consequences

- Playlists keep this for the first version: no manual Next or Previous, in the app or on the lock screen. Only automatic advance exists ([[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]). A manual Next and Previous are a later feature ([[US-0257 Go to the next or previous episode by hand]]).
- Revisit when playlists exist and "next episode" has a meaning
  ([[ADR-0006 No auto-advance until playlists exist]]).
