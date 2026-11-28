---
type: user-story
status: accepted
feature:
  - "[[playlists-editor]]"
related:
  - "[[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]"
  - "[[US-0248 Be warned when an episode I add is hidden by the rules]]"
---
As a listener
I want a remove button on the episodes I added by hand in the playlist preview
so that I can take one out without leaving the editor

Notes (later, not specified further):

- The button shows only on rows that came from the hand-picked list, and can be hidden (a preview option, see [[US-0246 Choose what the playlist preview shows]]).
- It does the same as **Remove from playlist** on the episode menu, and removes the episode's pin too if no other source provides the episode.
- To be done with the hand-picked list.
