---
type: feature
status: draft
priority: low
area:
---
# Search & Discovery

Allow user to find podcasts by searching, rather than requiring url (through
opml import, or raw "add feed"

Need to find sources/directories that I can use.

* Search by name
* Subscribe from search (or download single episodes)

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
