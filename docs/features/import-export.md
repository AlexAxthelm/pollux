---
type: feature
status: draft
priority: mid
area: subscription
depends:
  - "[[subscription]]"
  - "[[playlists]]"
---
# Import/export

Moving data in and out of Pollux.

## Parts

- [[import-export-opml]]: standard OPML import and export of subscriptions, including a subset
- [[import-export-playlists]]: a more detailed export that carries app-specific settings, for sharing a playlist

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
