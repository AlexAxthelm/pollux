---
type: feature
status: accepted
priority: high
area: playlists
parent: "[[playlists]]"
depends:
  - "[[playlists]]"
---
# Circular Dependencies in Playlists

By allowing playlists to include other playlists as sources, circular
dependencies are possible — a playlist including itself directly, or more likely
via indirection through other playlists.

## Why It's Usually Fine

Circular dependencies don't usually cause infinite loops in practice, and some users
may create them on purpose. The reasoning is in
[[ADR-0030 Playlist resolution is capped at 1,000 episodes instead of detecting cycles]].

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

## Decisions

- [[ADR-0030 Playlist resolution is capped at 1,000 episodes instead of detecting cycles]] (draft)

## User stories

```base
views:
  - type: table
    name: User Stories
    filters:
      and:
        - file.hasLink(this.file)
        - type == "user-story"
    order:
      - file.name
      - status
    sort:
      - property: file.name
        direction: ASC
```
