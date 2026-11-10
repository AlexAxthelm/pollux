---
type: feature
status: draft
priority: low
area: storage
parent: "[[storage]]"
depends:
  - "[[storage-limits]]"
  - "[[storage-eviction]]"
  - "[[downloads-auto]]"
  - "[[settings]]"
  - "[[playlists]]"
---
# Storage styles

A "storage style" is a preset that sets many storage- and download-related
settings at once, so the user doesn't have to think about individual settings.
Examples range from **full archive** (keep everything) to **just-in-time**
(episodes are downloaded on the fly, with the next few episodes kept available in
between).

A style is effectively a bundle of values for settings that live in
[[storage-limits]], [[storage-eviction]] and [[downloads-auto]].

## Behavior

- The user picks a style at the user level.
- The user-level style is available as a playlist setting too, so a playlist can
  use a different style from the default.
- The user can still tweak individual settings while otherwise using a style
  (fine control). A tweaked style should be shown as customized rather than
  silently becoming a different preset.

## Open questions

- Which presets ship, and what exactly does each one set?
- How is "customized" represented, and how does the user get back to the preset?
- How do style settings at the playlist level interact with user-level ones
  (same inheritance question as retention rule conflicts in
  [[storage-eviction]])?

## User stories

```base
views:
  - type: table
    name: User Stories
    filters:
      and:
        - file.hasLink(this.file)
        - type == "user-story"
    order:
      - file.name
      - status
    sort:
      - property: file.name
        direction: ASC
```
