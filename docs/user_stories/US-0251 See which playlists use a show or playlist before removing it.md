---
type: user-story
status: accepted
feature:
  - "[[playlists-editor]]"
  - "[[subscription-unsubscribe]]"
related:
  - "[[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]"
  - "[[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]"
---
As a listener
I want to be told which playlists use a playlist or show before I delete it or unsubscribe
so that I know what else will change

Notes (later, not specified further):

- The confirmation says how many playlists use it ("This playlist is included in 5 other playlists") and previews them.
- It applies to deleting a playlist that others include, and to unsubscribing from a show used as a source.
- On confirm the source is dropped from each of them, and the pins that came through it are removed.
