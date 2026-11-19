---
type: feature
status: draft
priority: low
area: player
depends:
  - "[[player]]"
---
# Sleep Timer

Allow users to set a timer after which playback stops.

## Timer Options

- Preset durations (e.g. 15 min, 30 min, 45 min, 1 hour)
- End of current episode
- End of current playlist
- Custom duration input

"End of playlist" and "End of episode" should display the remaining playlist time as context
(e.g. "End of playlist — 2hr 23min remaining")

PLayback stops with timer. By default, no fade out (option when setting timer)

## UI notes

not visible on mini-player.
visual affordance when active, but does not need detail (possible button goes
from outline to fill, or something like that)

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
