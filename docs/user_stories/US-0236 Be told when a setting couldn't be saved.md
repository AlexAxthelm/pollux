---
type: user-story
status: accepted
feature:
  - "[[settings-ui]]"
related:
  - "[[US-0166 Settings are persisted]]"
  - "[[US-0235 Be told when saved settings can't be read]]"
  - "[[ADR-0045 Settings are saved as one JSON document per level]]"
---
As a User
I want to be told when a setting I changed couldn't be saved
so that I'm not surprised when it reverts the next time I open the app

Note: the change stays in effect for the current run. The user may not be able to fix the cause, but should know. Related to [[US-0235 Be told when saved settings can't be read]].
