---
type: user-story
status: draft
feature:
  - "[[app-startup]]"
  - "[[library]]"
related:
  - "[[US-0238 The app opens straight into the library without a loading screen]]"
---
As a User
I want what is on screen to appear all at once, without the layout jumping as pieces arrive
so that I can read and tap without things moving under me

Note: data that is visible (for example the library list) loads in one go, with the UI blocked until it has. Data that isn't visible may load in the background, quickly. This matters less for local data, but is noted as a rule. Open: what counts as visible for a long list, and artwork that arrives after its row ([[app-startup]]).
