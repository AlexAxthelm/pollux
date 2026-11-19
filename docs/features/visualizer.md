---
type: feature
status: draft
priority: low
area: player
---
# Vizualizer

As an alternative to album art, there should be one or more Vizualizers
(spectrograph, FFT, etc)

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
