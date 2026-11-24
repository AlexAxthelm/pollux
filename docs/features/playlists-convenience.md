---
type: feature
status: accepted
priority: MVP
area: playlists
parent: "[[playlists]]"
depends:
  - "[[playlists]]"
  - "[[playlists-rules]]"
---
# Convenience playlists and quick options

## Convenience Playlists

System-provided playlists that require no configuration:
- **All subscriptions**: all episodes from all subscribed feeds, no filters
- **Downloaded / available**: all downloaded episodes, regardless of feed
- **Random All**: all episodes, random sort *(not MVP — depends on random sort
  in [[playlists-rules]] and [[random]])*

## Quick options

The stories ask for two related things, and the same solution may serve both:

- **Ready-made playlists** that don't need configuring, so the user can start listening
  quickly. The stories name "All subscriptions, Downloaded, in progress, etc."
- **Quick options** that give the user sensible starting points for building their own
  playlist.

Archive binge is surfaced as a suggested playlist from the create-playlist flow; see
[[playlists-archive]].

*This section is drawn from the stories below; it has not been specified further.*

## Open questions

- Is "in progress" a system playlist? The stories name it, but the list above does not. What it means is settled: an episode that is not played, with its playback position not at the start.

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
