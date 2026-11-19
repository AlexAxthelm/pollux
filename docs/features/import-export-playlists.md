---
type: feature
status: accepted
priority: mid
area: subscription
parent: "[[import-export]]"
depends:
  - "[[playlists]]"
  - "[[import-export-opml]]"
---
# Detailed export and sharing

> **Not built yet.** There is no export code in the app.

We would also look to have a more detailed export functionality that includes OPML plus some other information around actual application-specific settings — so that I would be able to share like a playlist, some smart playlist where it's like "all of the podcasts from these three feeds that have been released since 2024" or whatever, but that doesn't necessarily need to be OPML

- #needs-decision **What format is the detailed export?** The text above says it
  "doesn't necessarily need to be OPML", and nothing else decides it.

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
