---
type: feature
status: draft
priority: MVP
area: episode
depends:
  - "[[subscription]]"
---
# Episodes

Episodes are the atomic unit of content in Pollux.

## Parts

- [[episode-ui]]: the list row, swipe gestures and the detail page
- [[episode-chapters]]: chapters

## State

### Playback Status

A mutable field representing the user's current relationship with the episode:

- **Unplayed**: not yet started
- **In-progress**: started but not finished
- **Played**: completed (or manually marked as such)

In playlist filters, "unplayed" means not played, so it includes started episodes ([[playlists-rules]]).

Status is set automatically by playback (reaching the end of an episode, within a
small tolerance, marks it as played). Users may also set status manually at any
time, regardless of actual listening history.

How and when playback marks an episode played is in [[player-progress]] and
[[ADR-0002 Played is decided when the listener leaves an episode]].

Status is a field used for playlist filtering. Listening history (play counts, time
listened) is tracked separately — see [[listening-stats]].

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
  in it; may still be available locally or on other devices (see
  [[subscription-delisted-episodes]])

### Star / Flag

A user-set marker to distinguish an episode. Used to influence playlist and
download behavior (see [[playlists-rules]]), and to protect episodes from
eviction (see [[storage-protection]]). Additional flag types may be added later.

## Actions

### Context Menu (three-dots / long press)

- Play
- Download / Delete download
- Mark as played / Mark as unplayed
- Flag / Unflag *(star or equivalent)*
- Add to playlist (adds the episode to that playlist's hand-picked list, the same action as **+ Episode** in the playlist editor; not MVP, since hand-picked lists are not; [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]])
- Share link *(system share sheet)*
- Go to subscription / feed
- Move to top / Move to bottom *(only shown when viewing episode within a playlist; not MVP, it needs pins, [[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]])*
- *(future)* Add / remove tags

## UI

The list row, swipe gestures and detail page are in [[episode-ui]].

## Chapters

See [[episode-chapters]].

## Open Questions

- Whether the app can check if an episode is still available on the server without
  downloading it is tracked in [[storage-protection]].

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
