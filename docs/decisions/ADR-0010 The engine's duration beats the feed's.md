---
type: decision
status: accepted
feature:
  - "[[player-progress]]"
related:
  - "[[ADR-0002 Played is decided when the listener leaves an episode]]"
---
# The engine's duration beats the feed's

## Context

The feed's duration may be missing or wrong. A stream can be ready while its
length is still unknown, and learn it later with no further status change.

## Decision

The engine's duration is read when the item becomes ready and again whenever it
changes. It beats the feed's for the scrubber, seek limits and played detection.

## Consequences

- With no known duration, only the engine reaching the end finishes an episode
  (see [[ADR-0002 Played is decided when the listener leaves an episode]]).
