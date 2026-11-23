---
type: user-story
status: draft
feature:
  - "[[playlists-editor]]"
  - "[[playlists-rules]]"
related:
  - "[[US-0202 Add episodes to playlists]]"
  - "[[US-0242 Order a playlist by hand with top and end pools]]"
  - "[[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]"
---
As a curator
I want to see when a filter is hiding an episode I added by hand
so that I don't wonder why an episode I picked isn't in the playlist

Notes (not specified further):

- Hand-picked episodes are filtered like any other, so they can drop out.
- A natural place is the "Hand-picked" section of the editor, showing those episodes marked as filtered out, ideally with which filter did it.
