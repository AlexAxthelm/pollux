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
- Until then there is no skip. Marking the playing episode as played does not stop it: the audio keeps playing, and the listener has to scrub to the end or let it finish. What marking it played does is keep the playlist going, so the next episode plays when it ends ([[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]). If the app restarts before it finishes, the episode is not restored, because a played episode is not ([[ADR-0012 The play context is restored on launch with capped save retries]]). Kept as decided for the first version.
- To settle with this story: a manual Next near the end of an episode. The completion rule marks an episode played when the listener starts something else within the played tolerance ([[ADR-0002 Played is decided when the listener leaves an episode]]), so say whether Next follows that or is an exception that leaves the episode unplayed.
