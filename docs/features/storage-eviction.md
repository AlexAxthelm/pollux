---
type: feature
status: draft
priority: mid
area: storage
parent: "[[storage]]"
---
# Storage eviction

When automatic eviction is implemented, the proposed priority order (lowest to
highest protection) is:

1. **Evict first**: played episodes, no flag, oldest first
2. **Evict next**: unplayed episodes, oldest first
3. **Never evict**: flagged / starred episodes

All eviction rules should be user-configurable for power users.

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
