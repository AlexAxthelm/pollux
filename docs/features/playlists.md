---
type: feature
status: accepted
priority: MVP
area: playlists
depends:
  - "[[episode]]"
  - "[[subscription]]"
  - "[[playlists-rules]]"
  - "[[playlists-circular-dependencies]]"
---
# Playlists

Playlists are the primary way users organize and consume episodes in Pollux.

## Parts

- [[playlists-playback]]: playing a playlist, the active playlist, duplicates
- [[playlists-ui]]: the full and compact views
- [[playlists-convenience]]: system playlists and quick options
- [[playlists-rules]]: filters and sorts
- [[playlists-editor]]: creating and editing a playlist
- [[playlists-archive]]: archive binge playlists
- [[playlists-circular-dependencies]]: playlists that include each other
- [[playlists-how-it-works]]: a plain-language guide, a draft for the in-app help

## Data Model

A playlist is fundamentally a **query** over a set of sources, with filters and sorts applied:

```
(sources) → [rules: filters, sorts, deduplicate, in one ordered stack] → ordered episode list
```

**Sources** are shows, other playlists, the All shows source, and (later) the playlist's own hand-picked list; see [[ADR-0027 A playlist is a query over sources]].

**Filters** narrow which episodes are included (see [[playlists-rules]])

**Sorts** define the order of the resulting episodes (see [[playlists-rules]])

This model is intentionally complete from the start. MVP scope is limited by
which filter and sort *operations* are implemented, not by the model itself.

### Subscriptions as Implicit Playlists

A feed subscription behaves as an implicit single-feed playlist. It responds to
the same play/queue actions as any other playlist. This means playback logic
only needs to understand playlists — not whether the source is a subscription
or a user-created playlist.

## Meta-playlists

A playlist whose sources include other playlists inherits those playlists'
resolved episode lists at playback time. This is the primary mechanism for
building composite listening queues. See [[playlists-archive]].

## Circular Dependencies

See [[playlists-circular-dependencies]].

A playlist already on the resolution path is left out ([[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]), and each playlist
has an episode limit ([[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]]).

## MVP slice

The model is complete from the start; the slice is which parts get built first. The milestone is the one in [[ROADMAP]]: create a playlist with rules, play from it, and have it behave as the active playlist.

**In**

- **Sources:** subscriptions and other playlists. The **All shows** source is in, because the All Subscriptions convenience playlist is one; its entry in the editor's picker can follow.
- **Rules:** one stack with include/exclude, drag to reorder and remove, a result count under each rule and a line of helper text on the tie-break order ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]). Filters: "First N episodes", "N per source", "Playback status", Deduplicate (on by default, at most one), Title. Sorts: by age, by source. No sorts means source order.
- **Resolution:** the episode limit (under Advanced), and a playlist already on the path is omitted from resolution ([[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]), since both are needed for resolution to end.
- **Playback:** the playlist is the active context, with automatic advance on the re-resolved list, and an edit that removes the playing episode lets it finish ([[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]). This replaces [[ADR-0006 No auto-advance until playlists exist]]. The play context carries a `Playlist` source and a route.
- **Convenience playlists:** All Subscriptions, In progress.
- **Editor:** name, Save, Back with an unsaved-changes warning, Sources, Rules, Settings (Advanced switch, episode limit), a preview that matches the playlist view, and the empty state. Delete a playlist.
- **Playlist views:** full and compact ([[playlists-ui]]).

**Out (later)**

- The playlist's hand-picked list and everything that depends on it: [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]], Add to playlist, + Episode, [[US-0243 See when a filter hides a hand-picked episode]], [[US-0248 Be warned when an episode I add is hidden by the rules]], [[US-0252 Remove a hand-picked episode from the preview]].
- Pins and Manual ordering ([[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]], [[US-0242 Order a playlist by hand with top and end pools]]).
- The queue ([[ADR-0046 Queued episodes play before the active playlist resumes]]).
- Undo and redo ([[US-0245 Undo and redo in the playlist editor]]), the preview's details menu and Show excluded ([[US-0246 Choose what the playlist preview shows]], [[US-0247 Preview the rules at any point in the stack]]).
- Loop confirmation in the editor ([[US-0244 Be asked before making a playlist loop]]), the warnings in [[US-0250 Be warned before removing the playing episode from its playlist]] and [[US-0251 See which playlists use a show or playlist before removing it]] (plain deletes and unsubscribes still work, with the ordinary confirmation).
- The Downloaded convenience playlist (needs the Download status filter), the later filters and sorts in [[playlists-rules]], "Play next: from top", the archive template, import and export, playlist reordering.

**Settings in the slice.** The play context carries the route, and the settings resolver walks it ([[ADR-0044 An episode's settings come from the context it is played from]]). Playlist-level settings are wired up for a minimal set only, such as skip time; the other settings join as they are built on the settings branch. Building the resolver over the route is part of this slice. Settings not yet wired up fall through to the subscription and global levels, as they do today.

## Decisions

- [[ADR-0027 A playlist is a query over sources]]
- [[ADR-0028 A subscription is an implicit single-feed playlist]]
- [[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]]
- [[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]
- [[ADR-0046 Queued episodes play before the active playlist resumes]]
- [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]
- [[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]
- [[ADR-0043 Every filter can include or exclude]]

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
