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

## Data Model

A playlist is fundamentally a **query** over a set of sources, with filters and sorts applied:

```
(sources) → [filters] → [sorts] → ordered episode list
```

**Sources** may be:
- One or more feed subscriptions
- One or more other playlists (enabling composite / meta-playlists)
- A mix of both

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

**MVP behavior:** No detection required. Episode resolution is capped at 1,000
items. If the cap is reached, show: *"Showing first 1,000 episodes."*

The reasoning, and the fact that it is not yet confirmed, are in
[[ADR-0030 Playlist resolution is capped at 1,000 episodes instead of detecting cycles]].

## Decisions

- [[ADR-0027 A playlist is a query over sources]]
- [[ADR-0028 A subscription is an implicit single-feed playlist]]
- [[ADR-0030 Playlist resolution is capped at 1,000 episodes instead of detecting cycles]] (draft)
- [[ADR-0041 Queued episodes play before the active playlist resumes]]
- [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]
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
