# Library Recommendations
---
priority: mid
depends: subscription, playlists, listening-stats, sync
---

A recommendations section at the top of the library view, surfacing contextually
useful playlists and content to resume.

## Known Recommendation Types

- **Last played playlist**: quick resume of whatever was playing most recently
- **Continue on this device**: episodes in-progress
- **Continue from other devices**: episodes in-progress on other devices
  (depends on `sync.md`)
- **Playlists you play often**: frequently used playlists surfaced for quick
  access

## Behavior

- Not shown in MVP — library opens directly to subscription list
- Recommendations should update based on listening history
  (see `listening-stats.md`)

## Open Questions

- Should recommendations be dismissible / hideable?
- How many recommendations to surface at once?
- Should there be a way to pin a playlist to always appear here?
