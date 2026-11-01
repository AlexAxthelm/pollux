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
