---
type: user-story
status: accepted
feature:
  - "[[player]]"
  - "[[playlists-playback]]"
related:
  - "[[ADR-0046 Queued episodes play before the active playlist resumes]]"
---
As a listener
I want to queue an episode
so that it plays without changing the playlist I'm listening to 

Notes (later, not specified further):

- Add to queue is in an episode's "more" menu and adds to the end of the queue without interrupting the current episode; adding to the top is [[US-0260 Add an episode to the top of the queue]]. The Up next sheet shows the queue and the next few episodes of the playlist; reordering and removing are behind Edit; Clear asks for a press-and-hold confirmation that an app setting can turn off ([[ADR-0046 Queued episodes play before the active playlist resumes]]).
