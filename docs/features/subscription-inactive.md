---
type: feature
status: accepted
priority: mid
area: subscription
parent: "[[subscription]]"
depends:
  - "[[subscription]]"
---
# Inactive Subscriptions

There should be an indicator on subscriptions that are no longer publishing
(feed not updating, or no new episodes after some time limit). Configurable, but
default 6 months.

This should not in anyway change subscription behavior, if it's still available,
accessible (so archives are still accessible), but would be good for filtering
and searching, as well as general awareness that a show is no longer active

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
