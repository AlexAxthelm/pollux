---
type: user-story
status: accepted
feature:
  - "[[settings-ui]]"
related:
  - "[[US-0166 Settings are persisted]]"
  - "[[ADR-0045 Settings are saved as one JSON document per level]]"
---
As a User
I want to be told when my saved settings can't be read
so that I'm not surprised when the app uses defaults, and I don't lose my settings without knowing

Note: the app falls back to the defaults and leaves the unreadable document in place, so nothing is dropped quietly. It stays until the user changes a setting, which replaces it. A failed read (a database error) must not let the next change save over a good document. Whether the user is offered a way to keep, repair or discard the unreadable one is open.
