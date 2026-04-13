# Storage & Downloads
---
priority: MVP
depends: episode, subscription, settings, notifications
---

Pollux downloads episode audio files for offline playback. Storage management is
designed to be transparent and user-controlled, without silently deleting content.

## Storage Limit

The user may set a soft cap on how much local storage Pollux may use. This limit
is not reserved — other apps may use the same space freely. Pollux will simply
stop downloading new episodes once the limit is reached.

Display note: the UI should make clear that the limit is a cap, not a reservation.
Suggested copy: *"Pollux will not download new episodes once this limit is
reached, but this space is not reserved — other apps can use it freely."*

### Default Limit

No limit is set by default — Pollux may use all available device storage.

The app should nudge the user to set a limit at an appropriate moment (not
first run). Nudge intensity should increase when available device storage is low
(roughly under 4 GB free). See `notifications.md` for nudge implementation.

A suggested starting point when the user does set a limit: **max(4 GB, 20% of
device storage)**. At ~60 MB/hour (128kbps MP3), 4 GB ≈ 66 hours of content.

Note that the download limmit should be consluted **before** starting a
download (if the current space used + size of pending downloads + file in
consideration is bigger than the limit, do not download). If there are any
individual episodes that are bigger than the limit (small limit, or very big
episode files) then that should probably be a notification to the user (when
that gets implemented).

### Limit Configuration

The limit is set as a number in GB or hours (user picks unit; the other
representations update live):

- **GB**: direct storage size
- **Hours**: approximate playback time (~60 MB/hour at 128kbps; shown as an
  estimate, not a guarantee)
- **% of device**: read-only annotation, not an input mode

### Storage UI

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

## Download Behavior

### Fail on Full

When the download queue would push usage over the storage limit (or device
storage is exhausted), new downloads fail rather than evicting existing content.
The user is notified and must manually free space before downloads resume.

This is intentional — the app should never silently delete content the user
might care about.

*(Future)* Automatic eviction with user-configured priority rules. See eviction
notes below.

### Download Queue

- Episodes are queued for download and processed in order
- Parallel downloads are configurable (default 1)
- Queue is visible in the storage/download screen
- Queue items can be reordered (move to top) or cancelled
- Failed downloads show an error and a retry button

### What Gets Downloaded

**By default (subscription level):**
- Most recent episode, if unplayed

**Playlist-level download requests (later):**
- Ensure first N episodes are available
- Ensure at least X hours of playtime available
- Playlists may request additional downloads beyond subscription defaults
- Playlists may not prevent episodes from being downloaded

**Manual:**
- User explicitly downloads any episode via context menu or swipe gesture
- How does the download queue interact with background app refresh / OS
  download managers? Likely varies significantly by platform.


### Background task

If the platformOS supports background app refresh / tasks, then downlopad should
proceed with that. If not, then there should be a warning to user that it only
downloads while open.

### Parallel

If possible (within platform limits) downloads should be parallelized (up to
some limit, 4? 16?). For MVP, serial download is fine.

## Episode Availability & Protection

Episodes that have been removed from a feed's RSS but exist in local storage
should be flagged as "Removed from feed" and protected from automatic cleanup.

Flagged / starred episodes are never automatically deleted.

## Device specific

see storage-sync.md for device-specific rules
For MVP, storage settings should not sync.

## UI

### Download Screen

- Current download progress (if active), with progress bar
- Download queue list — reorderable, cancellable
- Failed downloads with error message and retry button

### Storage Screen

- Visual usage breakdown (see above)
- Storage limit input
- Per-subscription storage breakdown
- Option to manually delete downloaded episodes by subscription

## Open Questions

- Can the app check server availability of an episode without downloading it?
  (Needed to distinguish "removed from feed but still on server" from "gone
  entirely", and "listed in feed, but missing file" — relevant for archive protection.)
- Should there be a "download all episodes" option per subscription, with an
  appropriate warning about storage impact?
- Should the storage screen be part of settings, or accessible directly from the
  library / subscription view?

## Future: Eviction

see storage-eviction.md
