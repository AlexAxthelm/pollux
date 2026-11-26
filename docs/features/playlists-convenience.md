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
- **All subscriptions**: all episodes from all subscribed feeds, newest first, no filters. Playing from the library plays this playlist (a playlist whose only source is the All shows source, [[ADR-0050 A playlist with no sources is empty]])
- **Downloaded / available**: all downloaded episodes, regardless of feed (needs the Download status filter, [[playlists-rules]])
- **In progress**: episodes in progress, from all subscribed feeds (a playback status filter; the definition is in [[playlists-rules]])
- **Random All**: all episodes, random sort *(not MVP — depends on random sort
  in [[playlists-rules]] and [[random]])*

**What they are.** Convenience playlists are real playlists, so what works for a playlist works for them, except settings (see below): other playlists can use them as sources, playback and the queue can point at them, and they are listed in the library. Each has a fixed, well-known id, so that a device does not create a second copy and references stay stable. It is flagged as a system playlist, and its sources and rules are defined in the app, not stored, so a new version of the app can change them. They are **not editable, not deletable, not hideable and not renamable**. To change one, the user **duplicates** it: every playlist, system ones included, has a Duplicate action, and the copy is an ordinary user playlist that stores its own sources, rules and settings ([[US-0255 Duplicate a playlist]]). The system ones are then fixed starting points. In the first version only **All subscriptions** and **In progress** exist.

Convenience playlists have filters and sorts but no settings of their own, so an episode played from one resolves its settings from its show, then global, as it does when played from the library ([[ADR-0044 An episode's settings come from the context it is played from]]). Exceptions can be made if a need turns up.

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

None open.

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
