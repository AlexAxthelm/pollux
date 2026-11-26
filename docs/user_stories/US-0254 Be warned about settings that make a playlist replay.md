---
type: user-story
status: draft
feature:
  - "[[playlists-editor]]"
  - "[[playlists-playback]]"
related:
  - "[[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]"
  - "[[US-0219 Playlist from top of list]]"
---
As a listener
I want a warning when my playlist's settings and rules are likely to make it play the same episodes again and again
so that I can fix it before it happens

Notes (later, not specified further):

- The case in mind is a playlist set to play from the top that has episodes which never become played, or rules that keep bringing finished episodes back. Which combinations to warn about is to be worked out.
- The warning is a hint in the editor, not a block.
