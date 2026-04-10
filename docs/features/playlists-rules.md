# Playlist Rules
---
priority: high
depends: playlists
---

Rules are the filters and sorts applied to a playlist's sources to produce an ordered episode list. Rules are applied in sequence; order matters.

## UI

see `playlist-editor.md`

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
| By source (interleaved) | strategy: round-robin / proportional / date-normalized | Solves the archive-clumping problem; see `playlists-archive.md` |
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
