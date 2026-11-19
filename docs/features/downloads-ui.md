---
type: feature
status: accepted
priority: MVP
area: downloads
parent: "[[downloads]]"
depends:
  - "[[downloads-queue]]"
  - "[[downloads-failures]]"
---
# Downloads UI

## Download screen

- Current download progress (if active), with a progress bar
- Download queue list — reorderable, cancellable (see [[downloads-queue]])
- Failed downloads with error message and retry button (see
  [[downloads-failures]])

The screen is reachable from the [[library]]

## Download explanations

The user can see **why** a file is being downloaded (auto-download rule,
playlist request, next-to-play, or manual), so app behavior is understandable.
See [[downloads-auto]].

## Episode-level indicators

Per-episode status (not downloaded, downloading, downloaded, failed) and the
progress indicator are defined in [[episode]]

## Related

Storage usage and deleting downloaded files live in [[storage-ui]].

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
