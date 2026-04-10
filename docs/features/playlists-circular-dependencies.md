# Circular Dependencies in Playlists
---
priority: high
depends: playlists
---

By allowing playlists to include other playlists as sources, circular
dependencies are possible — a playlist including itself directly, or more likely
via indirection through other playlists.

## Why It's Usually Fine

Circular deps don't necessarily cause infinite loops in practice. The finite set
of subscribed episodes acts as a natural bound — filters further reduce the set.
In most cases, an episode appears only at its first position in the resolution
order, and subsequent circular inclusions add nothing new.

Some users may intentionally create circular playlists (e.g. a never-ending
shuffle of everything).

## MVP Behavior

No detection required. The app resolves episodes up to a cap of **1,000 items**.
If the cap is reached, the playlist displays:

*"Showing first 1,000 episodes."*

This handles all cases safely — intentional or accidental circular deps both
resolve to a finite, usable list.

Note that this should be configurable in advanced settings

## Future: Detection and Warning

A circular dependency warning in the playlist config UI would help users
understand and resolve unintentional loops. This requires cycle detection in
the playlist source graph and a clear, non-alarming way to surface it.

This is non-trivial to communicate well — defer until there's a clear UX
approach.
