---
type: feature
status: accepted
priority: mid
area: bookmarks
parent: "[[bookmarks]]"
depends:
  - "[[bookmarks]]"
  - "[[player]]"
  - "[[episode-ui]]"
---
# Bookmarks UI

How the user makes, finds and uses bookmarks.

> **Not built yet.** The episode detail view has a placeholder section that says
> "Bookmarks aren't built yet."

from a list of bookmarks, I should be able to click on one, and jump to that
part or the episode (intentional click).
maybe with press-and-hold plays a preview?

From a playing episode, I should be able to have a "quick bookmark" button that
makes a mark at the current timestamp

Perhaps there's an option that has a bookmark button on the home screen / lock
controls?

Would be good to have a view of bookmarks in an episode when seeing the episode
view, but also have a way to organize/filter across episode-bookmarks

- #needs-decision **Can the lock screen carry a bookmark button?** The text above wonders
  about "a bookmark button on the home screen / lock controls". [[player-os-integration]]
  lists the lock-screen commands as play, pause, toggle, skip and scrubbing, and has no
  bookmark command. Both are kept as written.

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
