---
type: feature
status: draft
priority: MVP
area: downloads
parent: "[[downloads]]"
depends:
  - "[[subscription]]"
  - "[[playlists]]"
  - "[[downloads-queue]]"
  - "[[storage-limits]]"
---
# Automatic and manual downloads

What gets downloaded, and who decides.

## Subscription level (default)

By default, the most recent episode of each show is downloaded automatically, if
unplayed, so the user doesn't need to do it themselves. The default rule for new
subscriptions is a setting (see `settings.md`).

New episodes are evaluated against the feed's download rules after a refresh;
refreshing itself is metadata-only and does not trigger downloads directly (see
`subscription.md`).

## Playlist level (later)

- Ensure the first N episodes are available
- Ensure at least X hours of playtime are available
- Playlists may request additional downloads beyond subscription defaults
- Playlists may not prevent episodes from being downloaded

## Manual

The user explicitly downloads any episode via context menu or swipe gesture (see
`episode.md`).

## Keeping the current episode ready

The episode the user was listening to should be immediately available to resume,
so they don't have to look for it.

## Interactions

- Downloads stop when the storage limit is reached; see [[storage-limits]] and
  [[storage-notifications]].
- Auto-deleting stale episodes is handled by [[storage-eviction]].
- Presets that set these rules in bulk are in [[storage-styles]].

## Open questions

- Should there be a "download all episodes" option per subscription, with an
  appropriate warning about storage impact? (The warning UI would live in
  [[storage-ui]].)

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
