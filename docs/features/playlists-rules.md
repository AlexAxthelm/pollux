---
type: feature
status: draft
priority: high
area: playlists
parent: "[[playlists]]"
depends:
  - "[[playlists]]"
---
# Playlist Rules

Rules are the filters and sorts applied to a playlist's sources to produce an ordered episode list. Rules are applied in sequence; order matters.

## UI

see [[playlists-editor]]

## Filters

Filters narrow the set of episodes included in the playlist.

### MVP

| Rule | Parameters | Notes |
|---|---|---|
| Limit by recency | N (count), direction (newest / oldest) | "Most recent 1" or "Oldest 1" are the same operation with different direction |
| Playback status | unplayed / in-progress / played (multi-select) | Enables "in progress" playlists, unplayed-only queues, etc. |

### Later

| Rule | Parameters | Notes |
|---|---|---|
| Limit total results | N (count) | Caps playlist length after other filters; high value for large archives |
| Date range | start / end date | "Episodes released after X" |
| Duration | min / max seconds | Short-form only, long-form only, etc. |
| Starred / favorited | — | Only episodes the user has flagged |
| Tag | tag name(s) | Depends on tagging feature |
| subscriptions | multi-select subscription | filter calculated playlists down to a set of subscriptions |

## Sorts

Sorts define the order of the filtered episode list.

### MVP

| Rule | Parameters | Notes |
|---|---|---|
| By age | asc / desc | Chronological or reverse-chronological |
| By source | ordered source list | All episodes from source A before source B; enables priority podcasts and composite playlist sequencing. Sequential only for MVP. |

### Later

| Rule | Parameters | Notes |
|---|---|---|
| By source (interleaved) | strategy: round-robin / proportional / date-normalized | Solves the archive-clumping problem; see [[playlists-archive]] |
| By duration | asc / desc | |
| By play count | asc / desc | Surfaces favorites or finds unlistened gems |
| Random | - | - |

## Rule Composition

Rules are stacked and applied in order. The UI should make reordering easy and show a live preview of the resulting episode list.

An empty source set with a rule applied acts on all episodes from all subscribed feeds.

A rule applied to a populated source set acts only on that set.

## Notes

- The rule system is intentionally analogous to a SQL SELECT: sources = FROM, filters = WHERE, sorts = ORDER BY
- Each new rule type can be added incrementally without changing the underlying data model

## Open questions

- #needs-decision **Include and exclude.** The filter tables above have no include or
  exclude. The editor spec puts an include/exclude toggle on every filter
  ([[playlists-editor]]), and the stories ask to include or exclude by starred status,
  date, duration, title and listened status
  ([[US-0221 Playlist filter for stars]], [[US-0222 Playlist filter for date]],
  [[US-0223 Playlist filter for duration]], [[US-0224 Playlist filter on titles]],
  [[US-0225 Playlist filter for status]]) and to exclude episodes that match some criteria
  ([[US-0215 Playlist episode exclusion]]).
- #needs-decision **Title filter.** [[US-0224 Playlist filter on titles]] asks for a title
  filter. It is not in the MVP or Later filter tables.
- Specific episodes can be added or included by hand
  ([[US-0202 Add episodes to playlists]], [[US-0216 Include episodes in playlist]]). How
  that relates to rules is not specified here; see the reordering question in
  [[playlists-playback]].

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
