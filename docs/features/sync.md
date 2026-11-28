---
type: feature
status: draft
priority: low
area: sync
depends:
  - "[[episode]]"
  - "[[playlists]]"
  - "[[subscription]]"
  - "[[bookmarks]]"
  - "[[settings]]"
---
# Sync

Sync settings across devices. Episodes (audio files) should be re-downloaded if
possible.

Also sync playback positions and event logs (played/unplayed, etc), playlists,
bookmarks, etc.

Sync is out of scope for MVP and v1.0.

## Parts

- [[sync-settings]]: whether settings sync or stay per-device
- [[sync-downloads]]: per-device control over which episodes are downloaded
- [[sync-playback]]: subscriptions, playback positions, the listening log, and per-device state

## Playlists

Playlists are one of the things that sync. Convenience playlists (All subscriptions, In progress) are system playlists: real rows with fixed, well-known ids, with their sources and rules defined in the app and not stored ([[playlists-convenience]]). Sync must treat them as system rows, so that each device does not end up with a second copy, and must not sync their definitions as user data. A user's playlists sync as sources plus one ordered rule stack, plus settings, pins and the hand-picked list ([[import-export-playlists]], which has the same question about what travels).

## Decisions

- [[ADR-0037 Audio files are re-downloaded on each device, not synced]]

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
