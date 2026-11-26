---
type: user-story
status: accepted
feature:
  - "[[playlists]]"
  - "[[playlists-circular-dependencies]]"
related: []
---
As a curator
I want to include other playlists
so that I can inherit their filters and sorts

Note: an included playlist contributes its resolved episode list (after its own rules), and the including playlist then applies its own rules to the union ([[ADR-0053 Resolution is bounded by the work it does, and playlists have no episode limit]]).