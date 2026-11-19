---
type: feature
status: accepted
priority: mid
area: subscription
parent: "[[import-export]]"
depends:
  - "[[subscription]]"
  - "[[playlists]]"
---
# OPML import/export

> **Not built yet.** There is no OPML code in the app.

Nothing too controversial on this one — pretty bog-standard OPML import/export. I think we would want to be able to, on export, select some subset of the known database of podcasts and only export those. Or maybe export all of the podcasts that are on a playlist.

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
