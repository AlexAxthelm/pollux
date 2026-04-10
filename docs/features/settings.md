# Settings
---
priority: MVP
depends: storage, player, episode, playlists, subscription, notifications
---

A central location for user preferences that affect behavior.
Accessible from a gear/settings icon in the library toolbar, or via contextual
entry points throughout the app.

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
- Storage limit (see `storage.md`)
- Parallel downloads (default: 1 for MVP)
- Default download rule for new subscriptions (default: most recent episode,
  if unplayed)

### Feed Refresh
- Global refresh interval (default: 12 hours)

### Gestures
- Swipe gesture mapping — configure per-gesture actions on episode rows
  (see `episode.md`)

### Feed settings

- A list of feeds (playlists and subscriptions) that when selected, take the
  user to the feed-specific settings page.

### Notifications
- *(to be defined — see `notifications.md`)*

### Appearance
- *(to be defined)*

### Advanced

Settings for power-users. Should be hidden by default, but can be shown

* Limit for playlist episodes (default 1000)


## Cascading Defaults Pattern

Many settings follow a cascading default pattern: a global default applies
unless overridden at a more specific level (episode → feed → global).

The UI should always make clear whether a value is using the global default or
a custom override, and offer a way to reset to default:

- `Refresh every 12 hours (default)` — using global default
- `Refresh every 24 hours (default)` — global default has been changed
- `Refresh every 1 hour` — per-feed override is set; subtle reset button shown

This pattern applies to: refresh interval, download rules, skip increments, and
any other setting that makes sense to configure globally but override locally.

## Per-Feed Settings

Per-feed settings are accessible from the subscription details page, not from
the main settings screen. They mirror global settings with override capability.
See `subscription.md`.

Per feed settings should be a separate page, accessible either from the main
settings page, or by clicking the "gear/settings" button from the feed details
page.

### Feed settings

sections inherit from global page, and use defaults (with a "(default)" note
unless changed. if not default, there should be a listtle "reset to default"
button next to it.
- Playback
- Library (display options)
- Storage (how many episodes to keep locally)

### Subscription settings

Same as feed settings, but also refresh interval

## Open Questions

- Should settings be synced across devices, or are they per-device?
  Storage limit is clearly per-device. Skip increments could go either way.
  See `sync.md`.
