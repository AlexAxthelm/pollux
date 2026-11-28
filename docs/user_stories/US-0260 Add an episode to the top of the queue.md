---
type: user-story
status: draft
feature:
  - "[[episode]]"
related:
  - "[[ADR-0046 Queued episodes play before the active playlist resumes]]"
  - "[[US-0152 Queue]]"
---
As a listener
I want to add an episode to the top of the queue
so that it plays right after the current episode, ahead of what I already queued

Notes (later, not specified further):

- **Add to queue** adds to the end of the queue and never interrupts the current episode ([[ADR-0046 Queued episodes play before the active playlist resumes]]). This is the same without the wait at the end of the line: the episode goes to the top, and the current episode still plays to its end.
- It is not called "Play next", because that is the playlist's next-episode setting ([[ADR-0044 An episode's settings come from the context it is played from]]); the label is to be chosen.
