---
type: decision
status: accepted
feature:
  - "[[playlists]]"
  - "[[playlists-circular-dependencies]]"
related:
  - "[[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]"
  - "[[ADR-0044 An episode's settings come from the context it is played from]]"
  - "[[US-0208 Playlists include other playlists]]"
---
# Playlist resolution is capped by a per-playlist episode limit

## Context

Playlists can include other playlists, and a playlist can be very large (a show's whole archive, or several). Resolution needs a bound so a playlist is always finite and cheap enough to resolve and preview. How loops between playlists are handled is a separate decision: [[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]].

## Decision

Every playlist is bounded by an **episode limit**.

- The limit is a setting, "Limit for playlist episodes": a global default of **1,000**, and a per-playlist override. Both are under Advanced settings ([[settings]]).
- There is also a hard maximum the setting cannot exceed, set to **50,000** for now. That number is provisional until it is tested for performance.
- The limit applies to the playlist's final list, after its own filters and sorts. If the limit cut the list, the playlist shows *"Showing first N episodes."* with its own N.
- An included playlist is resolved with **its own limit**. A playlist is the union of its sources' lists, each already limited, and then its own filters, sorts and limit are applied. Example: a playlist limited to 10,000 includes one limited to 2,000 and one limited to 5,000, so it holds up to 7,000 episodes if there are no duplicates.
- The limit is a property of a playlist's own resolution, not of playback, so it does not follow the context route of [[ADR-0044 An episode's settings come from the context it is played from]]. Its effective value is the playlist's override, else the global default.

## Consequences

- The hard maximum needs performance testing before it is final.
- The limit cuts the end of the list. With "send to end" pins ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]), an end-pinned episode can be cut when the limit is reached. This is accepted.
- An included playlist that was cut by its own limit is not marked as cut in the playlists that include it.
- With Deduplicate off, repeated copies count toward the limit.
