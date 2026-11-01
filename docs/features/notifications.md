---
type: feature
status: draft
priority: low
area: notifications
depends:
  - "[[storage]]"
  - "[[subscription]]"
---
# Notifications

Pollux should notify users of relevant events without being intrusive.

## Known Notification Types

- **Storage nudge**: prompt user to set a storage limit when none is configured,
  with increasing urgency as device free space decreases (see `storage.md`)
- **Download failed**: notify when a download fails due to storage full or
  network error
- *(more to be defined as features are specced)*

## Open Questions

- Which platforms / notification systems need to be supported?
- Should there be a global notification preference in settings?
- What is the right escalation model for the storage nudge?

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
