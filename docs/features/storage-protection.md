---
type: feature
status: draft
priority: mid
area: storage
parent: "[[storage]]"
depends:
  - "[[episode]]"
  - "[[subscription]]"
---
# Episode protection

Hard rules that keep certain episodes safe from automatic cleanup. These
override [[storage-eviction]] and retention rules.

## Removed from feed

Episodes that have been removed from a feed's RSS but exist in local storage
should be flagged as "Removed from feed" and protected from automatic cleanup.
The local copy may be the only one left.

## Flagged and starred

Flagged / starred episodes are never automatically deleted. (Flags are described
in `episode.md`.)

## Open questions

- Can the app check server availability of an episode without downloading it?
  (Needed to distinguish "removed from feed but still on server" from "gone
  entirely", and "listed in feed, but missing file" — relevant for archive
  protection.)
- Related: `subscription-soft-delete.md` and `playlists-archive.md` cover other
  cases where an episode's source goes away.

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
