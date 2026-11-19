---
type: feature
status: accepted
priority: MVP
area: storage
depends:
  - "[[episode]]"
  - "[[subscription]]"
  - "[[settings]]"
  - "[[notifications]]"
  - "[[downloads]]"
---
# Storage

Pollux keeps downloaded episode audio on the device. Storage management is
designed to be transparent and user-controlled, without silently deleting
content.

This epic covers how much space Pollux uses and what happens to files once they
are downloaded. How files get onto the device (the queue, background downloads,
auto-download, failures) is the [[downloads]] epic.

## Principles

- **The limit is a cap, not a reservation.** Other apps may use the space freely.
- **Fail on full, don't evict.** When storage is full, new downloads stop and the
  user is told; existing content is not deleted behind their back.
- **Deletion is surfaced.** Even when automatic eviction exists, it suggests
  before it deletes unless the user opts in to automatic action.
- **Protected episodes stay.** Flagged episodes and episodes removed from their
  feed are never cleaned up automatically.

## Parts

- [[storage-limits]] — the cap, its units, defaults, and enforcement
- [[storage-eviction]] — eviction and retention rules, auto-delete
- [[storage-protection]] — episodes that are always safe from cleanup
- [[storage-notifications]] — warnings, nudges, and "this could be deleted"
- [[storage-ui]] — usage breakdown, classification, bulk delete
- [[storage-styles]] — presets that bundle storage and download settings
- [[storage-sync]] — device-specific rules (storage settings do not sync for MVP)

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
