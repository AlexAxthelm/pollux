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

- Sequential playback through episodes, in playlist order. The playlist is resolved again each time playback moves on, and the next episode is the one after the episode that just finished, which counts as still in its place even if it has just been marked played. If an edit removes the playing episode, it finishes with the context it started with and playback then ends ([[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]])
    - Alternate behavior (configurable): play first unplayed episode in playlist
      on next-episode, rather than play next in sequence ("play from top" rather
      than "play next")
- Current position tracked (which episode, timestamp within episode)
- Pause and resume supported
- Episodes in a playlist may or may not be downloaded; download status visible
  per episode
- Duplicates are removed by the playlist's Deduplicate filter, which is on by default and
  can be moved or turned off ([[playlists-rules]]). With it off an episode can appear more
  than once, each time with its own settings route ([[ADR-0044 An episode's settings come from the context it is played from]])
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

**Episode ordering note:** The playlist's list is not dragged into order; episodes are pinned instead, and pinned episodes can be dragged within their group (later). Episodes are pinned to the top
or end with "Move to top / Move to bottom", which adds a "Manual ordering" sort to the rules (not MVP;
see [[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]] and [[US-0242 Order a playlist by hand with top and end pools]]).

**Queue:** A separate queue plays before the active playlist resumes, without
changing it. See [[ADR-0046 Queued episodes play before the active playlist resumes]].
*Not built yet.*

## Related

- The player has no auto-advance until playlists exist:
  [[ADR-0006 No auto-advance until playlists exist]].
- Playback remembers what it was started from:
  [[ADR-0007 Playback source is separate from the episode's feed]].

## Decisions

- [[ADR-0029 The active playlist is set by where playback started]]
- [[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]
- [[ADR-0046 Queued episodes play before the active playlist resumes]]
- [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]
- [[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]

## Open questions

Reordering by hand is decided in
[[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]; queueing
in [[ADR-0046 Queued episodes play before the active playlist resumes]], which leaves how items are added and cleared open.

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
