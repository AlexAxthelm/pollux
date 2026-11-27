---
type: user-story
status: draft
feature:
  - "[[playlists-playback]]"
  - "[[player]]"
related:
  - "[[ADR-0009 Next and previous track are disabled on the lock screen]]"
  - "[[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]"
---
As a listener
I want to move to the next or the previous episode of a playlist by hand
so that I can skip an episode I do not want without waiting for it to end

Notes (later, not specified further):

- Not in the first version: only automatic advance exists, and the lock screen keeps its skip buttons ([[ADR-0009 Next and previous track are disabled on the lock screen]]).
- A manual Next would work like an automatic advance (the playlist is resolved again and the episode after the current one plays), without marking the current episode played.
- A Previous needs a play history, because played episodes drop out of a playlist's list. That is its own piece of work.
- Until then the way past an unwanted episode is to mark it as played. Marking the playing episode played keeps the playlist going, so the next episode plays when it ends ([[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]). Kept as decided for the first version.
