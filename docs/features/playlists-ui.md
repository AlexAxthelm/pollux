---
type: feature
status: accepted
priority: MVP
area: playlists
parent: "[[playlists]]"
depends:
  - "[[playlists]]"
  - "[[episode-ui]]"
---
# Playlists UI

How a playlist is shown. The editor is in [[playlists-editor]]; the episode rows are
described in [[episode-ui]].

## UI

### Full View

```
[ Icon / Title ]   [ Summary stats ]   [ ⋮ config ]   (config menu: Edit, Duplicate, Delete; Duplicate is later)
────────────────────────────────────────────────────
Episode row
Episode row
...
```

Summary stats show:
- Episodes ready to play (downloaded) / total hours ready
- Total episodes / total hours
- (These may transition between each other on a timer)

### Compact View

For use in menus and selection flows:
- Icon (or tiled album art)
- Playlist title
- Episode count, playtime (ready / total)
- Playlist type indicator

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
