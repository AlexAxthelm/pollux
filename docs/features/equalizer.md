---
type: feature
status: draft
priority: low
area: player
---
# Equalizer / Volume

* Should offer a mechanism to adjust overall volume levels for the episode, and
  per frequency (equializer)
* Adjustable per episode, or per feed
* Include reasonable defaults
    * Maybe some good "smart" option, that seeks to have a good range

* This should also be a good way to do a general volume adjustment for podcasts
  that are generally too loud or quiet

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
