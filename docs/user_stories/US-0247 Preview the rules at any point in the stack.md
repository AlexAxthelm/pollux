---
type: user-story
status: draft
feature:
  - "[[playlists-editor]]"
related:
  - "[[US-0211 Playlist previews]]"
  - "[[US-0246 Choose what the playlist preview shows]]"
---
As a curator
I want to preview the list at any point in the rules stack
so that I can see the base layers the later rules build on and understand why the final list looks the way it does

Notes (later feature, not specified further):

- Showing the list as it would be if only the rules down to a chosen row had been applied.
- Rules are evaluated as if each one is applied to the result of the ones above it
  ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]).
