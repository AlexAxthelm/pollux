---
type: feature
status: draft
priority: low
area: notifications
parent: "[[notifications]]"
depends:
  - "[[notifications]]"
  - "[[subscription-refresh]]"
---
# New-episode notifications

Telling the user that new content has arrived: a notification per new episode, and a daily
summary.

> **Not built yet.** The app raises no notifications.

New episodes are found by [[subscription-refresh]]. Notifying while the app is closed
depends on background refresh; see
[[ADR-0022 Background refresh is requested every 3 hours and persists before it ends]].

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
