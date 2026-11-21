---
type: feature
status: accepted
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
Accessible from a gear/settings icon at the top right of the library screen, or via
contextual entry points throughout the app (subscription, playlist and episode settings
are added as they are built; see [[settings-cascade]]).

## Parts

- [[settings-cascade]]: global defaults, and overrides per subscription, playlist and episode
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

### Subscription and playlist settings

- A list of subscriptions and playlists that when selected, take the user to that
  one's settings page. Each page is also reached from a gear on its own page; see
  [[settings-cascade]].

### Notifications
- *(to be defined — see [[notifications]] and [[notifications-settings]])*

### Appearance
- *(to be defined)*

### Advanced

Settings for power-users. Should be hidden by default, but can be shown

* Limit for playlist episodes (default 1000)

## Open Questions

- Should settings be synced across devices, or are they per-device?
  Storage limit is clearly per-device. Skip increments could go either way.
  See [[sync-settings]].

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
