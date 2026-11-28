---
type: user-story
status: draft
feature:
  - "[[playlists-editor]]"
  - "[[episode]]"
related:
  - "[[US-0202 Add episodes to playlists]]"
  - "[[US-0243 See when a filter hides a hand-picked episode]]"
  - "[[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]"
---
As a curator
I want to be warned when I add an episode that the playlist's rules will hide
so that "Add to playlist" doesn't seem to do nothing

Notes (later, not specified further):

- For example, adding a played episode to a playlist that includes only episodes that are not played.
- A warning, not a block: hand-picked episodes are filtered like any other.
