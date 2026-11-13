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

- Revisit when playlists exist and "next episode" has a meaning
  ([[ADR-0006 No auto-advance until playlists exist]]).
