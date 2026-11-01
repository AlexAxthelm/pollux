---
type: feature
status: draft
priority: mid
area: subscription
depends:
  - "[[subscription]]"
  - "[[playlists]]"
---
# OPML import/export

Nothing too controversial on this one — pretty bog-standard OPML import/export. I think we would want to be able to, on export, select some subset of the known database of podcasts and only export those. Or maybe export all of the podcasts that are on a playlist.

We would also look to have a more detailed export functionality that includes OPML plus some other information around actual application-specific settings — so that I would be able to share like a playlist, some smart playlist where it's like "all of the podcasts from these three feeds that have been released since 2024" or whatever, but that doesn't necessarily need to be OPML

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
