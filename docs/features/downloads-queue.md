---
type: feature
status: accepted
priority: MVP
area: downloads
parent: "[[downloads]]"
depends:
  - "[[episode]]"
  - "[[storage-limits]]"
---
# Download queue

Episodes waiting to be downloaded, and how they are processed.

## Queue behavior

- Episodes are queued for download and processed in order.
- Queue items can be reordered (move to top) or cancelled, so the user can
  prioritize some files.
- Episodes in the queue or active playlist are downloaded with high priority, so
  they are available when the current episode finishes.
- Before each download starts, the storage limit is checked; see "Check before
  downloading" in [[storage-limits]]. A queue that would exceed the limit stops
  rather than evicting (see "Fail on full").
- The queue is visible to the user; see [[downloads-ui]].
- Failed downloads show an error and a retry button; see [[downloads-failures]].

## Cancelled and deleted downloads

A download the user cancelled or deleted is remembered, even after the app
restarts, so Pollux doesn't download it again behind the user's back or keep
retrying a bad file.

Playback never downloads against the listener's wishes: once a download is
cancelled or deleted, only the explicit Download action lifts that. Today this
memory is kept for the session only; see `player.md` ("Declined downloads and the
bad-file memory last for the session") for the current behavior and the open
question about making it durable.

## Parallel downloads

If possible (within platform limits) downloads should be parallelized, up to
some limit (4? 16?). The number of parallel downloads is configurable (default 1;
see `settings.md`). For MVP, serial download is fine.

## Open questions

- What is a sensible upper bound for parallel downloads, and does it vary by
  platform or connection type?

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
