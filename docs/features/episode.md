# Episodes
---
priority: MVP
depends: subscription
---

Episodes are the atomic unit of content in Pollux.

## State

### Playback Status

A mutable field representing the user's current relationship with the episode:

- **Unplayed**: not yet started
- **In-progress**: started but not finished
- **Played**: completed (or manually marked as such)

Status is set automatically by playback (reaching the end of an episode, within a
small tolerance, marks it as played). Users may also set status manually at any
time, regardless of actual listening history.

Status is a field used for playlist filtering. Listening history (play counts, time
listened) is tracked separately — see `listening-stats.md`.

### Replaying a Completed Episode

If a user begins playing an episode already marked as played, its status reverts
to in-progress automatically. Play count increments again when it is completed.

An episode can be marked as played multiple times in this way. Play counts do not
stack from manual status changes: if a user marks an episode as played manually,
the play count does not increment. Similarly, marking an episode as unplayed does
not affect the play count.

### Download Status

Tracks whether the episode audio file is available in local storage:

- **Not downloaded**: only metadata available
- **Downloading**: in progress
- **Downloaded**: available for offline playback ("ready to play")
- **Failed**: download attempted but unsuccessful
- **Removed**: episode was previously in a subscription feed but no longer appears
  in it; may still be available locally or on other devices

### Star / Flag

A user-set marker to distinguish an episode. Used to influence playlist and
download behavior (see `playlist-rules.md`), and to protect episodes from
eviction (see `storage.md`). Additional flag types may be added later.

## Actions

### Context Menu (three-dots / long press)

- Play
- Download / Delete download
- Mark as played / Mark as unplayed
- Flag / Unflag *(star or equivalent)*
- Add to playlist
- Share link *(system share sheet)*
- Go to subscription / feed
- Move to top / Move to bottom *(only shown when viewing episode within a playlist)*
- *(future)* Add / remove tags

### Swipe Gestures (on episode list row)

Swipe actions are user-configurable. Defaults:

| Gesture | Default Action |
|---|---|
| Short swipe left | Flag / Unflag |
| Short swipe right | Download / Delete |
| Long swipe left | Played / Unplayed |
| Long swipe right | *(unassigned)* |

Each action is a toggle based on current state.

## UI

### List View (single row)

Displayed when episodes appear in a playlist, subscription, or search results:

- Small episode art
- Play button
- Title
- Three-dots menu
- Played status indicator
- Download status indicator (progress bar while downloading)
- Duration
- Playback position (if in-progress)

### Detail View (full page)

Expanded view for a single episode:

- Album art
- Title, feed name
- Release date, duration
- Show notes
- Chapters (if available)
- Chapter art (if available)
- Bookmark list (see `bookmarks.md`)

## Chapters

see chapters.md 

## Open Questions

- Can the app check if an episode is still available on the server without
  downloading it? (Relevant for eviction and archive protection in storage.)
