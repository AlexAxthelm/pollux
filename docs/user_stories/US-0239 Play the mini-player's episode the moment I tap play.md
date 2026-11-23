---
type: user-story
status: draft
feature:
  - "[[app-startup]]"
  - "[[mini-player]]"
related:
  - "[[ADR-0008 The audio session is claimed only when audio is about to play]]"
  - "[[ADR-0012 The play context is restored on launch with capped save retries]]"
---
As a User
I want the episode shown in the mini-player to be cued up and ready
so that it starts the moment I tap play, and I can pick up where I left off without waiting

Note: ready by the time the play button is visible. Today the restored episode is not loaded into the audio engine until the first play, so the first press does that work. #needs-decision This conflicts with [[player-os-integration]], which creates the engine lazily; see [[app-startup]].
