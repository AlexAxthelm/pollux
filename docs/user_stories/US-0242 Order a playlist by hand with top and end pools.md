---
type: user-story
status: draft
feature:
  - "[[playlists]]"
  - "[[playlists-editor]]"
  - "[[playlists-playback]]"
related:
  - "[[US-0204 Episodes can be reordered in playlists]]"
  - "[[US-0202 Add episodes to playlists]]"
  - "[[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]"
---
As a curator
I want to send episodes to the top or the end of a playlist, and drag them into order within those groups
so that I can set exactly what plays first and last without ordering everything by hand

Notes (not specified further; not MVP):

- Everything else stays ordered by the playlist's rules. Only the pinned episodes are ordered by hand.
- Two groups of pinned episodes: sent to top and sent to end. In the editor preview, pinned rows get drag handles; the other rows get up/down buttons to send them to the top or end group. Pinning only orders episodes that are already in the playlist; adding episodes by hand is a separate source ([[US-0202 Add episodes to playlists]]).
- Possibly an "Edit order" button instead of always-visible handles.
- Whether a full manual total order is ever offered is left open.
