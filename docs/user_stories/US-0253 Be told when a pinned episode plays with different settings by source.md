---
type: user-story
status: draft
feature:
  - "[[playlists-editor]]"
related:
  - "[[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]"
  - "[[ADR-0044 An episode's settings come from the context it is played from]]"
  - "[[US-0242 Order a playlist by hand with top and end pools]]"
---
As a listener
I want a small warning when an episode I pinned comes from more than one source with different settings
so that I know it will play with the settings of the copy that was pinned, not necessarily the ones I expect

Notes (later, not specified further):

- A pin is keyed by the episode, and with Deduplicate off the pinned copy is the first occurrence in the list ([[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]). Each copy has its own route, so the resolved settings (for example skip time) can differ between copies ([[ADR-0044 An episode's settings come from the context it is played from]]).
- The warning appears on the pinned row in the preview, only when the copies' resolved settings actually differ. It is a hint, not a block.
- The wording and what it links to are not decided.
