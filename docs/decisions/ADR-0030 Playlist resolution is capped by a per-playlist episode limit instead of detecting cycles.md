---
type: decision
status: draft
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
- **A playlist already being resolved is left out.** While resolving a playlist, a source that is already on the current resolution path contributes nothing. Only that source is cut; siblings and everything else are kept. Example: A includes B and C, and B includes A, C and D. Resolving A resolves B with A on the path, so B's reference to A is cut and B gives C + D. A is then (C + D) + C, which is C, D once Deduplicate has run. A path cannot repeat a playlist, so resolution always ends.
- **The editor asks before a loop is created.** Adding a source that already includes this playlist, directly or through other playlists, stops with a confirmation (for example "[source] already includes this playlist. Including it makes a loop, and [source]'s copy of this playlist is left out. Add anyway?") that links to the documentation on cycles. See [[playlists-editor]] and [[US-0244 Be asked before making a playlist loop]].
- Duplicates are removed by the playlist's Deduplicate filter, which is on by default (see [[playlists-rules]]). With it on, a cycle adds nothing new. With it off, a cycle can fill the list with repeated copies up to the limit.

## Consequences

- Intentional and accidental cycles both resolve to a finite, usable list.
- Detection and a warning in the playlist config UI are deferred. They need cycle detection in the playlist source graph and a clear, non-alarming way to surface it, which is non-trivial to communicate well, so the spec defers it until there is a clear UX approach.
- The hard maximum needs performance testing before it is final.
- Because a playlist on the path is left out, an included playlist resolves differently inside a loop than on its own. B inside A leaves out A, while B on its own includes A. This only happens in loops, and the confirmation before creating one is the place to say so. Playlists that arrive through import or sync skip that confirmation, so the editor can also mark a source that sits in a loop as one that leaves something out; the wording is not settled.
- The limit cuts the end of the list. With "send to end" pins ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]), an end-pinned episode can be cut when the limit is reached. This is accepted.
