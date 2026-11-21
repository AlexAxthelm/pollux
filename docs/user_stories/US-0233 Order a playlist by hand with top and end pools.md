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
  - "[[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]"
---
As a curator
I want to send episodes to the top or the end of a playlist, and drag them into order within those groups
so that I can set exactly what plays first and last without ordering everything by hand

Notes (not specified further; not MVP):

- Everything else stays ordered by the playlist's rules. Only the pinned groups are ordered by hand.
- Three groups: sent to top, one-off additions, sent to end. In the editor preview, those rows get drag handles; the other rows get up/down buttons to send them to the top or end group.
- Possibly an "Edit order" button instead of always-visible handles.
- Whether a full manual total order is ever offered is left open.
