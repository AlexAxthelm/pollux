---
type: decision
status: accepted
feature:
  - "[[playlists-circular-dependencies]]"
  - "[[playlists-editor]]"
related:
  - "[[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]]"
  - "[[ADR-0027 A playlist is a query over sources]]"
  - "[[US-0208 Playlists include other playlists]]"
  - "[[US-0244 Be asked before making a playlist loop]]"
---
# Resolution leaves out a playlist already on the path, and the editor asks before a loop is made

## Context

Because playlists can include other playlists, loops are possible: a playlist including itself directly, or through other playlists. This was split out of [[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]], where it was first written down.

## Decision

- **A playlist already being resolved is left out.** While resolving a playlist, a source that is already on the current resolution path contributes nothing. Only that source is cut; siblings and everything else are kept. Example: A includes B and C, and B includes A, C and D. Resolving A resolves B with A on the path, so B's reference to A is cut and B gives C + D. A is then (C + D) + C, which is C, D once Deduplicate has run. A path cannot repeat a playlist, so resolution always ends. Because of that, a loop never repeats episodes, and the episode limit is a size bound, not what ends resolution.
- **The editor asks before a loop is created.** Adding a source that already includes this playlist, directly or through other playlists, stops with a confirmation (for example "[source] already includes this playlist. Including it makes a loop, and [source]'s copy of this playlist is left out. Add anyway?") that links to the documentation on loops. See [[playlists-editor]] and [[US-0244 Be asked before making a playlist loop]].

## Consequences

- Because a playlist on the path is left out, an included playlist resolves differently inside a loop than on its own. B inside A leaves out A, while B on its own includes A. This only happens in loops, and the confirmation before creating one is the place to say so. It also means the resolved list of an included playlist cannot always be reused as it is.
- Playlists that arrive through import or sync skip the confirmation, so the editor can also mark a source that sits in a loop as one that leaves something out. The wording is not settled.
- A circular playlist is not a way to make a repeating or never-ending list. The path cut contributes nothing for the second pass.
