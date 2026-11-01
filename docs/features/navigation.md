---
type: feature
status: draft
priority: MVP
area:
depends:
  - "[[library]]"
  - "[[player]]"
  - "[[mini-player]]"
---
# Navigation Structure

Pollux has a minimal top-level navigation model:

- **Library** (home) — always accessible
- **Now Playing** — accessible via mini-player tap when something is active
- **Settings** — accessible from contextual menus throughout the app, not a
  top-level destination
- **Downloads** — accessible from the library view, not a top-level destination

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
