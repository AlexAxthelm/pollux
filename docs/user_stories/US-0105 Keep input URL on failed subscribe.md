---
type: user-story
status: draft
feature:
  - "[[subscription]]"
related: []
---
As a user
I want the URL I typed to still be there when a subscribe attempt fails
so that I can correct it instead of retyping it

Note: story is a concrete current gap: `ContentView.swift` clears the text field immediately on tapping Add, so a failed attempt loses what was typed.
Note: related to [[ROADMAP]] phase 5 - invalid URI on subscribe