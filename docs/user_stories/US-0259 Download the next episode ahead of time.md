---
type: user-story
status: draft
feature:
  - "[[downloads-queue]]"
  - "[[playlists-playback]]"
related:
  - "[[US-0049 Prioritize downloading next to play]]"
---
As a listener
I want the next episode of the playlist I'm listening to to be downloaded if it isn't yet
so that playback is not interrupted when the current episode ends

Notes (later, not specified further):

- Activating a playlist adds nothing to the download queue beyond the normal download and retention rules ([[downloads-queue]]). This story is the exception for the one episode that plays next.
- The next episode is whatever the playlist would play after the current one ([[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]), which can change while the current episode plays.
- How far ahead to download (one episode, or a short window) and what happens when storage is full are to be settled with the story ([[storage-limits]]).
