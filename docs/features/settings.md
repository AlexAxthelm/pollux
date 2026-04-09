# Settings
---
priority: MVP
---

A central location for user preferences that affect app-wide behavior.

## Known Settings

- **Storage limit**: cap on local storage usage (see `storage.md`)
- **Swipe gesture mapping**: configure per-gesture actions on episode rows
  (see `episode.md`)
- **Parallel downloads**: number of simultaneous downloads (see `storage.md`)
- **Playback behavior**: "play next" vs "play from top" on episode completion
  (see `playlists.md`)
- **Skip increment**: seconds for forward/back skip buttons (see `player.md`)
- *(more to be defined as features are specced)*

## Open Questions

- Should settings be a single screen or organized into sections?
- Are any settings per-device vs synced across devices (see `sync.md`)?

## Cascading Defaults Pattern

Many settings follow a cascading default pattern: a global default applies
unless overridden at a more specific level (e.g. per-feed, per-episode). In that
sense, episodes should inherit from feed, which inherit from subscription, then from global

The UI should always make clear whether a value is the global default or a
custom override, and offer a way to reset to default. Example display:

- `Refresh every 12 hours (default)` — using global default
- `Refresh every 24 hours (default)` — global default has been changed
- `Refresh every 1 hour` — per-feed override is set, a subtle "reset" button
  should be visible

This pattern applies to: refresh interval, download rules, and any other
setting that makes sense to configure globally but override locally.
