---
type: feature
status: draft
priority: MVP
area: settings
depends:
  - "[[storage]]"
  - "[[player]]"
  - "[[episode]]"
  - "[[playlists]]"
  - "[[subscription]]"
  - "[[notifications]]"
---
# Settings

A central location for user preferences that affect behavior.
Accessible from a gear/settings icon in the library toolbar, or via contextual
entry points throughout the app.

## Parts

- [[settings-cascade]]: global defaults, and overrides per feed, subscription, playlist and episode
- [[settings-ui]]: finding, searching, undoing and understanding settings

## Global Settings Screen Structure

Settings are organized into sections on a single scrollable screen.

### Playback
- Skip increment — forward (default: 30s)
- Skip increment — back (default: 15s)
- Resume behavior — rewind on resume (default: 3s; can be set to 0 to disable)
- Next episode behavior — play next in sequence vs. play first unplayed
  (default: play next)

### Library
- Unplayed count display — total unplayed vs. immediately available unplayed
  (default: total)

### Downloads & Storage
- Storage limit (see [[storage-limits]])
- Parallel downloads (default: 1 for MVP; see [[downloads-queue]])
- Default download rule for new subscriptions (default: most recent episode,
  if unplayed; see [[downloads-auto]])

### Feed Refresh
- Global refresh interval (default: 12 hours)

### Gestures
- Swipe gesture mapping — configure per-gesture actions on episode rows
  (see [[episode-ui]])

### Feed settings

- A list of feeds (playlists and subscriptions) that when selected, take the
  user to the feed-specific settings page.

### Notifications
- *(to be defined — see [[notifications]])*

### Appearance
- *(to be defined)*

### Advanced

Settings for power-users. Should be hidden by default, but can be shown

* Limit for playlist episodes (default 1000)

## Open Questions

- Should settings be synced across devices, or are they per-device?
  Storage limit is clearly per-device. Skip increments could go either way.
  See [[sync]].

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
