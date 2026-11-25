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
- Next episode behavior — "Play next" (in sequence) vs. "From the top" (default: play next). Resolves for the active context only, then global; it does not walk the route ([[ADR-0044 An episode's settings come from the context it is played from]])

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

* Episode limit (default 1000, per-playlist override, hard maximum 50,000 for now; see [[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]])

## As built

The global Settings screen is reached from the gear in the library toolbar. Of the sections
above, only three **Playback** settings exist so far:

- **Skip forward** (10, 15, 30, 45, 60 or 90 s; default 30 s)
- **Skip back** (same choices; default 15 s)
- **Rewind on resume** (Off, 3, 5 or 10 s; default 3 s)

Each row is a picker. A setting the user hasn't set reads "(default)"; one they have set shows
a "Reset to 30 s" button instead. Choosing the default's own number is still a choice, so it
counts as set (see [[settings-cascade]]). The core accepts whole seconds within a range (a skip of 1 to 3600, a rewind of 0 to 600; a saved
value outside it is treated as unset, and a change sent from the app outside it is ignored), and
the screen only offers the presets.

- **The first press waits for them.** At launch, a play, skip or scrub pressed before the saved settings
  have been read is held for a moment and then runs with them, not the defaults. 
  Blocking is okay here, because the expected block is on the order of milliseconds.
  If the read fails, it runs with the defaults.
- **The player follows them.** The skip buttons and their labels, the rewind when resuming and
  the rewind when starting an episode all read the settings. A change applies at once, to the
  episode that is playing.
- **Saved as JSON.** Settings are saved as one document per level, written by the core and
  stored by the shell as an opaque string. Only values the user has set are saved, and the
  document is read key by key, so one bad value doesn't cost the others. Keys this build
  doesn't know are kept when saving. See [[ADR-0045 Settings are saved as one JSON document per level]].
- **Subscription settings.** A gear on the subscription's page (next to Sort) opens **Podcast
  settings**, headed by the show's artwork and name and "Changes here apply to this podcast
  only", so it can't be mistaken for the library settings. It has the same three Playback rows.
  A value the podcast hasn't set shows the global value it follows and reads
  "(inherited from library)", not "(default)"; one it has set shows a reset button that returns
  it to the global value ("Reset to 45 s"). A change to a global setting shows through every
  podcast that hasn't set its own. See [[settings-cascade]] and [[subscription]].
- **The player resolves along the stack.** For each episode it reads its own subscription's
  settings first, then the global ones ([[ADR-0044 An episode's settings come from the context it is played from]]).
  A podcast's skip or rewind beats the global one; a value the podcast leaves unset follows it.
  Checked in the simulator: the player's buttons read −10 and +60 with the podcast set to 10 and
  60 s, and fell back to the global values when reset.
- **Not built yet:**
  - the other sections (next-episode behavior, library, downloads and storage, refresh interval,
    gestures, notifications, appearance, advanced) and the **Advanced** filter
    ([[US-0233 Advanced settings depend on what is being edited]]);
  - playlist and episode settings pages, and playlists on the context stack;
  - a list of subscriptions and playlists on the global page (see Subscription and playlist
    settings above); a subscription's page is reached only from its own page for now;
  - the **Advanced** filter on a subscription's page: none of the three settings is advanced yet;
  - the subscription settings that aren't playback (refresh interval, download rules);
  - dropping a subscription's settings in the core when unsubscribe is built
    ([[subscription-unsubscribe]]);
  - telling the user when saved settings can't be read or a change couldn't be saved
    ([[US-0235 Be told when saved settings can't be read]], [[US-0236 Be told when a setting couldn't be saved]]).
    Until then a failed save is silent and the change is lost at the next launch. After a failed
    read, changes are kept in memory and the read is retried each time the app comes to the
    foreground; once it succeeds they are saved, and if it keeps failing they last for the session only;
  - search, export and the new-setting notices ([[settings-ui]]);
  - a VoiceOver pass on the screens ([[settings-ui]]).

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
