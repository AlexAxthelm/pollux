---
type: feature
status: accepted
priority: MVP
area: playlists
parent: "[[playlists]]"
depends:
  - "[[playlists]]"
  - "[[player-progress]]"
---
# Playing a playlist

How a playlist plays, and how one becomes the active playlist. The data model is in
[[playlists]].

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

## Related

- The player has no auto-advance until playlists exist:
  [[ADR-0006 No auto-advance until playlists exist]].
- Playback remembers what it was started from:
  [[ADR-0007 Playback source is separate from the episode's feed]].

## Decisions

- [[ADR-0029 The active playlist is set by where playback started]]

## Open questions

- #needs-decision **Can episodes be reordered by hand?** The text above says "There is
  no drag-to-reorder. To pin specific episodes to the front, a rule should be used."
  The context menu in [[episode]] offers "Move to top / Move to bottom" when viewing an
  episode within a playlist, and [[US-0204 Episodes can be reordered in playlists]] asks to
  reorder specific episodes (its note leans toward an episode-specific rule). Both are kept
  as originally written.
- #needs-decision **How does queueing an episode fit?**
  [[US-0152 Queue]] asks to queue an episode so it plays without changing the playlist
  being listened to. The spec says activating a playlist evicts the previous one and has no
  queue.

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
