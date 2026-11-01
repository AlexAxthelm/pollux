---
type: feature
status: draft
priority: MVP
area: subscription
parent: "[[subscription]]"
depends:
  - "[[subscription]]"
---
# Unsubscribing

Unsubscribe is a destructive action and should be protected accordingly:
- Not in the primary UI — accessed via a menu or settings within the subscription
  details page
- Should warn about consequences (downloaded files, playlist membership, etc.)

When the user unsubscribes, the app should warn them that downloaded episodes
will be deleted (including those on other devices if known), espescially if any
have been flagged to be saved ("this will delete 27 episodes, including 3 saved
ones")

When a subscription is removed, playlists should have their rule re-evaluated,
so that the removal is reflected in those too.

see subscription-soft-delete.md for future unsubscribe improvements

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
