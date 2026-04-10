# Playlists
---
priority: MVP
depends: episode, subscription, playlist-rules, playlists-circular-dependencies
---

Playlists are the primary way users organize and consume episodes in Pollux.

## Data Model

A playlist is fundamentally a **query** over a set of sources, with filters and sorts applied:

```
(sources) → [filters] → [sorts] → ordered episode list
```

**Sources** may be:
- One or more feed subscriptions
- One or more other playlists (enabling composite / meta-playlists)
- A mix of both

**Filters** narrow which episodes are included (see `playlist-rules.md`)

**Sorts** define the order of the resulting episodes (see `playlist-rules.md`)

This model is intentionally complete from the start. MVP scope is limited by
which filter and sort *operations* are implemented, not by the model itself.

### Subscriptions as Implicit Playlists

A feed subscription behaves as an implicit single-feed playlist. It responds to
the same play/queue actions as any other playlist. This means playback logic
only needs to understand playlists — not whether the source is a subscription
or a user-created playlist.

## Behavior

- Sequential playback through episodes, in playlist order
    - Alternate behavior (configurable): play first unplayed episode in playlist
      on next-episode, rather than play next in sequence ("play from top" rather
      than "play next")
- Current position tracked (which episode, timestamp within episode)
- Pause and resume supported
- Episodes in a playlist may or may not be downloaded; download status visible
  per episode
- An episode may appear in a playlist more than once if explicitly added by the
  user; otherwise duplicates are suppressed (configurable)
- A playlist with no rules applied includes all episodes from its sources

**Playlist persistence:** The active playlist persists through fully closing the
app and reappears on next open. The active playlist becomes inactive only when it
reaches its end.

## Active Queue

There is one active playlist at a time. Activating a new playlist evicts the
previous one.

The active context is determined by **where in the UI the user initiated
playback**, not by which playlists happen to contain that episode:
- Playing from a playlist view → that playlist becomes active
- Playing from a subscription view → that subscription (implicit playlist) becomes active
- Playing from the library → the library (implicit all-subscriptions playlist)
  becomes active

**Activating a playlist:**
- Pressing play on an episode within a playlist view sets that playlist as active
  and begins playback at that episode
- Pressing play on a playlist from the library resumes from the current position
  if already active, or begins from the first episode if not

**Episode ordering note:** There is no drag-to-reorder. To pin specific episodes
to the front, a rule should be used. This will be a later rule type.

## Convenience Playlists

System-provided playlists that require no configuration:
- **All subscriptions**: all episodes from all subscribed feeds, no filters
- **Downloaded / available**: all downloaded episodes, regardless of feed
- **Random All**: all episodes, random sort *(not MVP — depends on random sort
  in `playlist-rules.md` and `random.md`)*

## Meta-playlists

A playlist whose sources include other playlists inherits those playlists'
resolved episode lists at playback time. This is the primary mechanism for
building composite listening queues. See `playlists-archive.md`.

## Circular Dependencies

See `playlists-circular-dependencies.md`.

**MVP behavior:** No detection required. Episode resolution is capped at 1,000
items. If the cap is reached, show: *"Showing first 1,000 episodes."*

## UI

### Full View

```
[ Icon / Title ]   [ Summary stats ]   [ ⋮ config ]
────────────────────────────────────────────────────
Episode row
Episode row
...
```

Summary stats show:
- Episodes ready to play (downloaded) / total hours ready
- Total episodes / total hours
- (These may transition between each other on a timer)

### Compact View

For use in menus and selection flows:
- Icon (or tiled album art)
- Playlist title
- Episode count, playtime (ready / total)
- Playlist type indicator
