---
type: user-story
status: draft
feature:
  - "[[app-startup]]"
  - "[[player]]"
related:
  - "[[ADR-0008 The audio session is claimed only when audio is about to play]]"
---
As a User
I want playback never to start just because I opened or returned to the app, and to carry on if it was already playing
so that sound only happens when I have asked for it

Note: opening the app, from a cold start or from the background, never starts audio by itself. Returning to an app that is already playing leaves it playing. As far as I know this is how the app behaves now: a cold relaunch came back paused in the simulator, and the docs say the same ([[app-startup]]). Returning to an app that is already playing has not been checked. The story makes it a requirement so it can't change by accident.
