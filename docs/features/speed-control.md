---
type: feature
status: draft
priority: low
area: player
depends:
  - "[[player]]"
---
# Playback Speed Control

Allow users to adjust playback speed.

## Behavior

- Continuous slider with soft snapping to common presets
- User can set arbitrary speed (e.g. 1.4x) but UI gently suggests nearby
  preset (e.g. 1.5x)
- Common presets: 0.5x, 0.75x, 1x, 1.25x, 1.5x, 1.75x, 2x
- Accessible from player options menu
- Should follow cascading defaults pattern (global → feed)
- Per feed settings is possible 
- "smart" speedups (eliminating silence) is a separate feature (much later)

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
