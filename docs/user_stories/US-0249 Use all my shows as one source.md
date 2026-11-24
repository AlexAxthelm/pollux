---
type: user-story
status: draft
feature:
  - "[[playlists-editor]]"
  - "[[playlists-convenience]]"
related:
  - "[[US-0200 Playlists have multiple subscriptions]]"
  - "[[ADR-0050 A playlist with no sources is empty]]"
---
As a curator
I want to add all my shows to a playlist as one source
so that I don't pick them one by one, and shows I subscribe to later are included

Notes (not specified further):

- It is the first item in the editor's list of shows.
- It follows the library: new subscriptions join and unsubscribed shows drop out.
- The "All subscriptions" convenience playlist is a playlist with only this source.
- Per-source rules treat it as one source for each show ([[ADR-0050 A playlist with no sources is empty]]).
