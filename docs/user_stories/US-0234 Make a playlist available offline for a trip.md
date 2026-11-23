---
type: user-story
status: draft
feature:
  - "[[settings-cascade]]"
  - "[[playlists]]"
  - "[[storage-eviction]]"
related:
  - "[[US-0051 Playlist storage style]]"
  - "[[US-0044 Retention rules interactions]]"
---
As a curator
I want to make everything in a playlist available offline, even though its shows keep only one episode or none
so that I can listen on a road trip without a connection

Note: the playlist asks for retention and downloads like a subscription would; storage combines the requests (default: retain). See [[ADR-0044 An episode's settings come from the context it is played from]].
