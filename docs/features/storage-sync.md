---
type: feature
status: draft
priority: high
area: storage
parent: "[[storage]]"
---
# Device-specific storage rules

Touches on sync (when we get to that), but there should be a way to have the
download rules/limits be device specific (sync can suggest that they are the
same, but not a strong suggestion). So I may want to have this feed downloaded
on my phone, but not my laptop, or vice versa. When sync works, it would be good
to have a way to see something like "available on phone" as a bonus feature

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
