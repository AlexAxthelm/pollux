---
type: feature
status: accepted
priority: MVP
area: subscription
parent: "[[subscription]]"
depends:
  - "[[subscription]]"
  - "[[episode]]"
  - "[[subscription-refresh]]"
---
# De-listed episodes

*Status: the marking is implemented; the hiding and toggle below are not.*

When an episode is no longer present in a feed's RSS/Atom/JSONFeed:

- Episode metadata is retained permanently in the local database
- Episode is hidden from the default subscription view
- A "Show unavailable episodes" toggle reveals them
- If the episode audio file is still on device, it remains playable
- Episode is marked with "Removed from feed" status (see `episode.md`)

Permanent metadata retention is intentional — it supports the library model
and keeps the archive complete even as publishers rotate content.

## Marking an episode removed

An episode is marked `RemovedFromFeed` only once it has been **continuously absent
for 48 hours of wall time**; a single absence is not enough, so a truncated or stale
response can't mark a feed removed. Only `NotDownloaded` or `Failed` episodes are
flagged: a downloaded, queued or in-flight episode keeps its state so its file stays
playable. A marked episode that reappears returns to `NotDownloaded`, and a refresh
that returns an empty feed marks nothing. A 304 can flag an episode whose clock has
already run out, but never starts a clock.

The full rules and the reasons are in
[[ADR-0023 An episode is marked removed only after 48 hours of continuous absence]].

## Related

- Protection of removed episodes from automatic cleanup: [[storage-protection]]
- Episode status: [[episode]]
- Stored fields (`missing_since`): `DATA_MODEL.md`

## Not yet

The episode list does not yet hide removed episodes or offer the "Show unavailable
episodes" toggle.

## Decisions

- [[ADR-0023 An episode is marked removed only after 48 hours of continuous absence]]

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
