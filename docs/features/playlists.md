# Playlists
---
priority: MVP
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

This model is intentionally complete from the start. MVP scope is limited by which filter and sort *operations* are implemented, not by the model itself.

### Subscriptions as Implicit Playlists

A feed subscription behaves as an implicit single-feed playlist. It responds to the same play/queue actions as any other playlist. This means playback logic only needs to understand playlists — not whether the source is a subscription or a user-created playlist.

## Behavior

- Sequential playback through episodes, in playlist order
    - Alternate behavior (configurable): play first unplayed episode in playlist
      on next-episode, rather than play next in sequence (i.e. "play from top"
      rather than "play next")
- Current position tracked (which episode, timestamp within episode)
- Pause and resume supported
- Episodes in a playlist may or may not be downloaded; download status is visible per episode
- An episode may appear in a playlist more than once if explicitly added by the user; otherwise duplicates are suppressed (configurable)
- A playlist with no rules applied includes all episodes from its sources

## Active Queue

There is one active playlist at a time. Activating a new playlist evicts the previous one.

The active context is determined by **where in the UI the user initiated playback**, not by which playlists happen to contain that episode. For example:
- Playing an episode from a playlist view → that playlist becomes active
- Playing an episode from a subscription view → that subscription becomes active
- Playing an episode from the library → the library (implicit all-subscriptions playlist) becomes active

Since the UI does not present episode listings for multiple playlists simultaneously, each play action has an unambiguous source context.

**Activating a playlist:**
- Pressing play on an episode within a playlist view sets that playlist as active and begins playback at that episode
- Pressing play on a playlist from the library resumes from the current position if already active, or begins from the first episode if not

**Episode ordering note:** There is no drag-to-reorder in the playlist view. To pin specific episodes to the front of a playlist, a rule should be used (e.g. "episode X, then episode Y, then everything else"). This will be a later rule type.

## Convenience Playlists

System-provided playlists that require no configuration:
- **All subscriptions**: all episodes from all subscribed feeds, no filters
- **Downloaded / available**: all downloaded episodes, regardless of feed
- **Random All**: All episodes, sorted randomly

## Meta-playlists

A playlist whose sources include other playlists inherits those playlists' resolved episode lists at playback time. This is the primary mechanism for building composite listening queues (e.g. "latest episodes first, then archives").

## Circular dependencies

By allowing the user to include other playlists as sources/feeds, the
possibility of circular dependencies is very real (a playlist includes itself as
a source, or more likely it includes another playlist that includes it, some
indirection to make it not obvious). I don't expect this to be a major problem,
as the subscriptions impose a finite set of episodes to draw from, which would
be further reduced with filters. In most situations, an episode will only appear
at its first position in the playlist, so later (circular) inclusions should be
a non-issue, but they may exist (perhaps intentionally for a never-ending
playlist). For playlists that may be unbounded the simple answer is probably to
only calculate the first N (1000? 10000?) items, and if there's something beyond
that, display a little message along the lines of "... and more" with a note
about the circular deps. Simirlarly, as a later feature, a circular dep warning
would be useful, but requires more thought on how to surface to the user in a
good way.

## UI

### Full View

```
[ Icon / Title ]   [ Summary stats ]   [ ⋮ config ]
────────────────────────────────────────────────────
Episode row
Episode row
Episode row
...
```

Summary stats show:
- Episodes ready to play (downloaded) / total hours ready
- Total episodes / total hours
- (These may transition between each other on a timer, as described in product notes)

### Compact View

For use in menus and selection flows:
- Icon (or tiled album art)
- Playlist title
- Episode count, playtime (ready / total)
- Playlist type indicator
