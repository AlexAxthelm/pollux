---
type: decision
status: accepted
feature:
  - "[[player-progress]]"
related:
  - "[[ADR-0002 Played is decided when the listener leaves an episode]]"
---
# Position persistence cadence and resume rewind

## Context

The engine reports the position about once a second. Writing every tick would be
wasteful, but too few writes lose the listener's place. Resuming exactly where
playback stopped also loses context.

## Decision

**Position persistence.** Written at most every 10s while playing, and on pause,
seek, backgrounding, switching episode, and end. Per-second ticks are transient.

**Resume rewind.** 3s on any resume after a pause (and on cold-start restore and
when resuming a saved position), clamped at 0.

## Consequences

- The rewind is hard-coded until Settings exists (`defaults.rs`), as are the
  30s/15s skips. The feature spec says the user should be able to disable the
  rewind; see [[player-progress]] and `settings.md`.
