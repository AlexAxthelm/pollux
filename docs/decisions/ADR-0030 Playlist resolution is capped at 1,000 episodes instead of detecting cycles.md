---
type: decision
status: accepted
feature:
  - "[[playlists-circular-dependencies]]"
related:
  - "[[US-0208 Playlists include other playlists]]"
---
# Playlist resolution is capped at 1,000 episodes instead of detecting cycles

## Context

Because playlists can include other playlists, circular dependencies are possible: a playlist including itself directly, or more likely through other playlists.

Circular dependencies don't necessarily cause infinite loops in practice. The finite set of subscribed episodes is a natural bound, and filters reduce it further. In most cases an episode appears only at its first position in the resolution order, and later circular inclusions add nothing new. Some users may intentionally create circular playlists (for example a never-ending shuffle of everything).

## Decision

No cycle detection for MVP. Episode resolution is capped at **1,000 items**. If the cap is reached, the playlist shows *"Showing first 1,000 episodes."*

The cap should be configurable in advanced settings ("Limit for playlist episodes", default 1000; see [[settings]]).

## Consequences

- Intentional and accidental cycles both resolve to a finite, usable list.
- Detection and a warning in the playlist config UI are deferred. They need cycle detection in the playlist source graph and a clear, non-alarming way to surface it, which is non-trivial to communicate well, so the spec defers it until there is a clear UX approach.
