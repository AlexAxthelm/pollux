---
type: feature
status: draft
priority: MVP
area: downloads
depends:
  - "[[episode]]"
  - "[[subscription]]"
  - "[[settings]]"
  - "[[storage]]"
  - "[[notifications]]"
---
# Downloads

Pollux downloads episode audio files for offline playback. Downloading is
designed to be transparent and user-controlled: the user can see what is queued
and why, cancel or reorder it, and is told when something fails or stops.

How much *space* downloads may use, and what happens to files afterwards, is the
[[storage]] epic. This epic covers how files get onto the device.

## Parts

- [[downloads-queue]] — the queue, ordering, cancelling, and parallel downloads
- [[downloads-background]] — downloading while the app is suspended or closed
- [[downloads-auto]] — what is downloaded without the user asking
- [[downloads-failures]] — failure notification, retry, and stopped downloads
- [[downloads-ui]] — the download screen and explanations

Related elsewhere:

- Authenticated feeds (protected downloads): [[subscription-auth]]
- Per-device download control: [[sync]] and [[storage-sync]]
- Download status on an episode: `episode.md`
- Streaming while a download is in flight: `player.md`

## Open questions

- How does the download queue interact with background app refresh / OS download
  managers? Likely varies significantly by platform. (See
  [[downloads-background]].)

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
