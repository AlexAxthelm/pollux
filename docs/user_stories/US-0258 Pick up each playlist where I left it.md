---
type: user-story
status: draft
feature:
  - "[[playlists-playback]]"
related:
  - "[[ADR-0029 The active playlist is set by where playback started]]"
  - "[[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]"
---
As a listener
I want each playlist and show to remember where I left it
so that pressing play on it picks up that episode, not the first one in its list

Notes (later, not specified further):

- In the first version, pressing play on a playlist or show that is not active starts at the first episode of its list, played or not, so a half-played episode in the middle is not picked up ([[ADR-0029 The active playlist is set by where playback started]]).
- This needs a stored position for each playlist and show: the episode, and where in it. Only one playlist is active at a time, so the others would need their last position kept when they are evicted.
- Whether the position survives edits to the playlist, and what happens when the remembered episode has left the list, are to be settled with the story.
