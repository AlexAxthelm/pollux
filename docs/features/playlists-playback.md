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

**Episode ordering note:** There is no drag-to-reorder. Specific episodes are pinned
with a playlist's hand-picked list: each member sits in a top, one-off or end group, and
the "Move to top / Move to bottom" menu actions set it (not MVP; drag within the top and
end groups is a later touch, see [[US-0242 Order a playlist by hand with top and end pools]]). See [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]].

**Queue:** A separate ephemeral queue plays before the active playlist resumes, without
changing it. See [[ADR-0046 Queued episodes play before the active playlist resumes]].
*Not built yet.*

## Related

- The player has no auto-advance until playlists exist:
  [[ADR-0006 No auto-advance until playlists exist]].
- Playback remembers what it was started from:
  [[ADR-0007 Playback source is separate from the episode's feed]].

## Decisions

- [[ADR-0029 The active playlist is set by where playback started]]
- [[ADR-0046 Queued episodes play before the active playlist resumes]]
- [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]

## Open questions

None open. Reordering by hand is decided in
[[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]; queueing
in [[ADR-0046 Queued episodes play before the active playlist resumes]].

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
