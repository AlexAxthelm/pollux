---
type: user-story
status: draft
feature:
  - "[[playlists-ui]]"
  - "[[playlists-convenience]]"
related:
  - "[[US-0214 Ready-made playlists]]"
  - "[[ADR-0027 A playlist is a query over sources]]"
---
As a listener
I want to duplicate a playlist, including the ready-made ones
so that I can change my own copy and start from something that already works

Notes (later, not specified further):

- Every playlist has a **Duplicate** action in its menu, system playlists too. The copy is a normal playlist the user owns and can edit, rename and delete.
- The copy keeps the original's sources (an included playlist is still included, not copied; the All shows source stays All shows), rules, settings and episode limit. For a playlist with a hand-picked list and pins, those are copied too.
- The copy's name starts as the original's name with "copy" added.
- This is how a system playlist is customised: the system playlists themselves are not editable ([[playlists-convenience]]). The archive template could use the same path.
