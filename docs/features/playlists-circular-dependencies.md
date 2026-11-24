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

## Resolution

While a playlist is being resolved, a source already on the current path contributes
nothing; only that source is cut. If A includes B and C, and B includes A, C and D, then A is
(C + D) + C. This always ends, and a loop never repeats episodes. See
[[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]].

The episode limit (default 1,000, per playlist) is a separate size bound; see
[[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]].

## Confirmation when adding a source

When the user adds a source that already includes this playlist (directly or through other
playlists), the editor asks first, says what will be left out, and links to this page. See
[[playlists-editor]] and [[US-0244 Be asked before making a playlist loop]]. *Not built yet.*

## Future: Marking existing loops

A persistent mark in the sources list for a source that sits in a loop, for playlists that
arrive by import or sync and skip the confirmation. The wording needs care so it isn't
alarming; not specified yet.

## Decisions

- [[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]
- [[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]]

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
