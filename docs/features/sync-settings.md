---
type: feature
status: draft
priority: low
area: sync
parent: "[[sync]]"
depends:
  - "[[sync]]"
  - "[[settings]]"
---
# Settings sync

> **Not built yet.** Sync is out of scope for MVP and v1.0.

Whether and how settings sync across devices. The stories below are both accepted and
point in different directions.

- #needs-decision **Do settings sync, or are they per-device?** [[US-0087 Sync settings]]
  wants settings to sync across devices, and [[US-0090 Per-device settings]] wants them
  controlled per device. Both are accepted. [[settings]] asks "Should settings be synced
  across devices, or are they per-device?" and notes that the storage limit is clearly
  per-device. [[storage-sync]] says storage settings should not sync in MVP. All are kept
  as written.

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
