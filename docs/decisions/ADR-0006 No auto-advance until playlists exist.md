---
type: decision
status: accepted
feature:
  - "[[player-progress]]"
  - "[[playlists]]"
related:
  - "[[US-0139 Auto play next episode]]"
---
# No auto-advance until playlists exist

## Context

Only subscription-as-playlist exists in the MVP build, so there is no defined
"next episode" to advance to.

## Decision

At the end of an episode it is marked played and the player goes inactive (the
mini-player disappears). "Next episode" arrives with playlists.

## Consequences

- This is a temporary decision. Supersede it when playlist playback lands
  ([[US-0139 Auto play next episode]]).
