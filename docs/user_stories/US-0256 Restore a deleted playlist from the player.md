---
type: user-story
status: draft
feature:
  - "[[player]]"
  - "[[subscription-soft-delete]]"
related:
  - "[[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]"
---
As a listener
I want the "From:" label of an episode from a deleted playlist to take me to that playlist, with a way to restore it
so that deleting a playlist I'm listening to is not a one-way action

Notes (later, not specified further; to do when soft-delete is built):

- Until then the label shows the deleted playlist's name and is not tappable ([[player]]).
- Tapping it opens the deleted playlist with a warning such as "This playlist has been deleted. Restore?".
- Restoring brings back the playlist as it was, including its sources, rules, settings, pins and hand-picked list ([[subscription-soft-delete]] for the grace period idea).
