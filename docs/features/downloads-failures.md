---
type: feature
status: draft
priority: mid
area: downloads
parent: "[[downloads]]"
depends:
  - "[[downloads-queue]]"
  - "[[notifications]]"
---
# Download failures

What happens when a download doesn't complete.

## Telling the user

- Failed downloads show an error and a retry button (see [[downloads-ui]]).
- The user is notified when a download fails or an episode cannot be downloaded
  (network error, storage full), so they can decide whether to try again.
- Downloads auto-stopped by the storage limit are explained to the user; see
  [[storage-notifications]] and "Fail on full" in [[storage-limits]].
- Authentication failures on protected feeds are surfaced rather than failing
  silently; see [[subscription-auth]].

## Retrying

- Failed downloads are automatically retried in a limited way, so transient
  errors (e.g. going offline) don't bother the user.
- A download that failed is retried when the user plays the episode again, even
  if it was replacing a file that turned out to be missing, so the episode ends
  up downloaded without the user having to find the Download button. The playback
  side of this is described in `player.md` ("A replacement download that failed
  is not retried on play").
- Downloads the user cancelled or deleted are not retried; see
  [[downloads-queue]].

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
