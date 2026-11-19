---
type: feature
status: draft
priority: low
area: notifications
parent: "[[notifications]]"
depends:
  - "[[notifications]]"
  - "[[settings]]"
---
# Notification settings

What the user controls about notifications: which ones are raised, when they are raised,
and the OS-level controls.

> **Not built yet.** The app raises no notifications.

- **Which notifications** ([[US-0082 Notification settings]]): by type.
- **When** ([[US-0083 Notification settings (time)]]): behavior, such as no notifications
  between 9pm and 7am, or batching.
- **OS controls** ([[US-0085 OS notification controls]]): the app cannot change what the OS
  allows. This can only mean pointing the user to the OS notification settings for the app.
  The story is kept as written.

The open question "Should there be a global notification preference in settings?" is in
[[notifications]].

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
