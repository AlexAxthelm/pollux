---
type: feature
status: accepted
priority: mid
area: bookmarks
depends:
  - "[[episode]]"
  - "[[player]]"
---
# Bookmarks

Allow user to mark specific timestamps (for later refernece).
May optionally have a text note attached to the bookmark. 

May lay the foundation for later features like sharing (at point) or looping
playback (a/b loops)

Effectively, just mark a position in an episode.

Don't worry about DAI (dynamic ad insertion) yet, but note it as a future
complication. For now, just timestamp on currently available episode.

## Parts

- [[bookmarks-ui]]: making, finding and jumping to bookmarks
- [[bookmarks-screenshots]]: creating a bookmark from a screenshot of the player

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
