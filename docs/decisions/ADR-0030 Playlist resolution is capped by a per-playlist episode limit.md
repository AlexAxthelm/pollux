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

- The limit is a setting, "Episode limit": a global default of **1,000**, and a per-playlist override. Both are under Advanced settings ([[settings]]).
- There is also a hard maximum the setting cannot exceed, set to **50,000** for now. That number is provisional until it is tested for performance.
- The limit applies to the playlist's final list, after its own filters and sorts. If the limit cut the list, the playlist shows *"Showing first N episodes."* with its own N.
- An included playlist is resolved with **its own limit**. A playlist is the union of its sources' lists, each already limited, and then its own filters, sorts and limit are applied. Example: a playlist limited to 10,000 includes one limited to 2,000 and one limited to 5,000, so it holds up to 7,000 episodes if there are no duplicates.
- The limit is a property of a playlist's own resolution, not of playback, so it does not follow the context route of [[ADR-0044 An episode's settings come from the context it is played from]]. Its effective value is the playlist's override, else the global default.

- Subscriptions (and the All shows source) are not capped. Only a playlist has an episode limit, until a reason turns up to cap them too.

- **Resolution has a work bound that is separate from the episode limit.** The episode limit caps a playlist's output, not the work to produce it, and resolution also runs when the player is restored at launch, where a runaway resolution would hang the app. So a resolution counts the playlists it expands, and when the count passes a fixed bound it stops expanding further included playlists and the playlist shows "Some included playlists were not fully loaded", in the same way as "Showing first N episodes." The number is chosen when the resolver is built, by measuring. Caching playlists that are outside any loop is an optimisation for the build, not a rule here. How the cost grows, and what to decide when import is built, is in [[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]], which is left as it is.

## Consequences

- The hard maximum needs performance testing before it is final.
- The limit cuts the end of the list. With "send to end" pins ([[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]), an end-pinned episode can be cut when the limit is reached. This is accepted.
- An included playlist that was cut by its own limit is not marked as cut in the playlists that include it.
- With Deduplicate off, repeated copies count toward the limit.
