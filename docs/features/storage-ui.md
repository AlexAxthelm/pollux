---
type: feature
status: draft
priority: MVP
area: storage
parent: "[[storage]]"
depends:
  - "[[storage-limits]]"
  - "[[episode]]"
  - "[[subscription]]"
---
# Storage UI

How the user sees, understands and manually manages what Pollux has stored on
the device. The download queue and progress live in [[downloads-ui]].

## Usage breakdown

A visual breakdown (bar graph or treemap) of current usage, colored by
subscription. Shows:

- Per-subscription usage
- Total used / limit (e.g. "4.2 GB / 42% of limit")
- Device free space (read-only, for context)

Suggested display:

```
[ sub A ][ sub B ][ sub C ][ free ]|← limit
4.2 GB used · 5.8 GB remaining · 9.3% of device (107 GB free)
```

## Storage screen

- Visual usage breakdown (see above)
- Storage limit input (see [[storage-limits]])
- Per-subscription storage breakdown
- Option to manually delete downloaded episodes by subscription

## Classifying episodes

So the user can decide what to keep, episodes in the storage view can be
inspected by:

- **Show** — episodes coded/grouped by subscription
- **Size** — sorted by file size
- **Date** — sorted by date

## Deleting

- **Bulk delete** — select many episodes and delete their downloads at once.
- **Delete all** — remove all downloaded episodes. This should be scopable or
  filterable (by show, size, date) rather than all-or-nothing.

Deleting a download is remembered so it is not silently brought back; see
[[downloads-queue]].

## Open questions

- Should the storage screen be part of settings, or accessible directly from the
  library / subscription view?
- Should there be a "download all episodes" option per subscription, with an
  appropriate warning about storage impact? (Tracked in [[downloads-auto]]; the
  warning UI would live here.)

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
