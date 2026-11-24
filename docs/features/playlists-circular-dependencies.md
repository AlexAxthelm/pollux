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
[[ADR-0030 Playlist resolution is capped by a per-playlist episode limit instead of detecting cycles]].

## MVP Behavior

No detection required. Each playlist resolves up to an episode limit (default **1,000**,
per-playlist override, hard maximum 50,000 for now). If the limit is reached, the playlist
displays:

*"Showing first N episodes."*

An included playlist is resolved with its own limit, and the including playlist is the union of
its sources, then its own rules and limit.

This handles all cases safely — intentional or accidental circular deps both
resolve to a finite, usable list.

The limit is set under advanced settings. How resolution stops for a real cycle is open; see
the ADR.

## Resolution

While a playlist is being resolved, a source already on the current path contributes
nothing; only that source is cut. If A includes B and C, and B includes A, C and D, then A is
(C + D) + C. This always ends. See the ADR below.

## Confirmation when adding a source

When the user adds a source that already includes this playlist (directly or through other
playlists), the editor asks first, says what will be left out, and links to this page. See
[[playlists-editor]] and [[US-0244 Be asked before making a playlist loop]]. *Not built yet.*

## Future: Marking existing loops

A persistent mark in the sources list for a source that sits in a loop, for playlists that
arrive by import or sync and skip the confirmation. The wording needs care so it isn't
alarming; not specified yet.

## Decisions

- [[ADR-0030 Playlist resolution is capped by a per-playlist episode limit instead of detecting cycles]] (draft)

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
