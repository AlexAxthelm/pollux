---
type: decision
status: accepted
feature:
  - "[[playlists-circular-dependencies]]"
related:
  - "[[US-0208 Playlists include other playlists]]"
---
# Playlist resolution is capped by a per-playlist episode limit instead of detecting cycles

## Context

Because playlists can include other playlists, circular dependencies are possible: a playlist including itself directly, or more likely through other playlists.

Circular dependencies don't necessarily cause infinite loops in practice. The finite set of subscribed episodes is a natural bound, and filters reduce it further. In most cases, with duplicates suppressed, an episode appears only at its first position in the resolution order, and later circular inclusions add nothing new. Some users may intentionally create circular playlists (for example a never-ending shuffle of everything).

## Decision

No cycle detection for MVP. Every playlist is bounded by an **episode limit**, so a playlist is always finite.

- The limit is a setting, "Limit for playlist episodes": a global default of **1,000**, and a per-playlist override. Both are under Advanced settings ([[settings]]).
- There is also a hard maximum the setting cannot exceed, set to **50,000** for now. That number is provisional until it is tested for performance.
- The limit applies to the playlist's final list, after its own filters and sorts. If it is reached, the playlist shows *"Showing first N episodes."* with its own N.
- An included playlist is resolved with **its own limit**. A playlist is the union of its sources' lists, each already limited, and then its own filters, sorts and limit are applied. Example: a playlist limited to 10,000 includes one limited to 2,000 and one limited to 5,000, so it holds up to 7,000 episodes if there are no duplicates.
- The limit is a property of a playlist's own resolution, not of playback, so it does not follow the context route of [[ADR-0044 An episode's settings come from the context it is played from]]. Its effective value is the playlist's override, else the global default.
- Duplicates are removed by the playlist's Deduplicate filter, which is on by default (see [[playlists-rules]]). With it on, a cycle adds nothing new. With it off, a cycle can fill the list with repeated copies up to the limit.

## Consequences

- Intentional and accidental cycles both resolve to a finite, usable list.
- Detection and a warning in the playlist config UI are deferred. They need cycle detection in the playlist source graph and a clear, non-alarming way to surface it, which is non-trivial to communicate well, so the spec defers it until there is a clear UX approach.
- The hard maximum needs performance testing before it is final.
- #needs-decision **How does resolution stop for a real cycle?** Under the union model, resolving A needs B, which needs A, and so on, so a per-playlist limit alone does not end the recursion. Some rule is needed. One option is that a playlist reached again while it is already being resolved contributes nothing. That is a local cut and needs no warning or UI, but it is a small form of cycle detection. Not decided.
- #needs-decision **The limit cuts the end of the list.** With "send to end" pins ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]), an end-pinned episode is the first to fall off when the limit is reached. Not decided.
