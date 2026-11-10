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

## Per-device downloads

Which episodes are downloaded is a per-device decision. A user with multiple
devices should be able to control which episodes get downloaded to each (for
example, a feed downloaded on the phone but not on the laptop, or vice versa), so
they can manage listening on each device.

Sync may suggest that download rules match across devices, but not strongly. Once
sync works, a view like "available on phone" would be a nice bonus. Storage
limits and other device-specific rules are in [[storage-sync]]; the download
machinery is in [[downloads]].

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
