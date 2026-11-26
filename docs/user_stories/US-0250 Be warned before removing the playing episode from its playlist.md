---
type: user-story
status: accepted
feature:
  - "[[playlists-editor]]"
  - "[[playlists-playback]]"
related:
  - "[[US-0245 Undo and redo in the playlist editor]]"
  - "[[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]"
---
As a listener
I want to be warned when an edit to the playlist I'm listening to would remove the episode that is playing
so that I know playback will end after it, and can undo the edit if I didn't mean it

Notes (later, not specified further):

- The warning says what will happen: the episode keeps playing with the settings it started with, and playback ends when it finishes ([[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]).
- It mentions that the editor's undo button can revert the edit ([[US-0245 Undo and redo in the playlist editor]]).
- It is a warning, not a block.
