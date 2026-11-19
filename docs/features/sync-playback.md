---
type: feature
status: draft
priority: low
area: sync
parent: "[[sync]]"
depends:
  - "[[sync]]"
  - "[[player]]"
  - "[[subscription]]"
  - "[[library-recommendations]]"
---
# Playback and library sync

> **Not built yet.** Sync is out of scope for MVP and v1.0.

Syncing subscriptions, playback positions and the listening log, and seeing what is
playing on each device. "Continue from other devices" in [[library-recommendations]] depends
on this.

- #needs-decision **Is the playback position shared, or per device?**
  [[US-0088 Sync playback]] wants the playback position to sync across devices.
  [[US-0092 Per-device playhead]] wants it to point to what was last played on this
  device. [[US-0089 Sync per-device state]] asks to see what is playing on each device and
  choose. Nothing says how these fit together. All are kept as written.

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
