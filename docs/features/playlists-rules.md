---
type: feature
status: accepted
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

Filters narrow the set of episodes included in the playlist. Every filter can include or
exclude ([[ADR-0043 Every filter can include or exclude]]).

### MVP

| Rule | Parameters | Notes |
|---|---|---|
| Limit by recency | N (count), direction (newest / oldest) | "Most recent 1" or "Oldest 1" are the same operation with different direction |
| Playback status | unplayed / in-progress / played (multi-select) | Enables "in progress" playlists, unplayed-only queues, etc. In-progress means not played, with the playback position not at the start. |
| Deduplicate | none | Keeps the first occurrence of each episode at this point in the rules. On by default, at the top of the filters; it can be moved or turned off. It has no include/exclude toggle. See [[US-0217 Playlist deduplication]], [[US-0218 Playlist allows duplicates]]. |
| Title | text | Includes or excludes episodes whose title contains the given text (a simple match). Not in the original MVP scope, but cheap to include. A general regular-expression filter may follow. See [[US-0224 Playlist filter on titles]]. |

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
| By source | none | Groups episodes by the order of the Sources section: all episodes from source A before source B; enables priority podcasts and composite playlist sequencing. Sequential only for MVP. |

### Later

| Rule | Parameters | Notes |
|---|---|---|
| Manual ordering | none | Appears when episodes are pinned to the top or end; pinned episodes are moved to the front or back in hand-set order. Starts at the bottom of the stack, so the pins hold; it can be moved. See [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]. |
| By source (interleaved) | strategy: round-robin / proportional / date-normalized | Solves the archive-clumping problem; see [[playlists-archive]] |
| By duration | asc / desc | |
| By play count | asc / desc | Surfaces favorites or finds unlistened gems |
| Random | - | - |

## Rule Composition

Rules are stacked and applied in order. The UI should make reordering easy and show a live preview of the resulting episode list.

Filters, sorts and Deduplicate share one stack and can be interleaved ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]). Each rule is applied to the list as if all the rules above it had already been evaluated, so with several sorts the last one wins and the ones above only break ties. For example, `sort: newest first`, `filter: first N episodes` (the "limit total results" filter, a later rule), `sort: oldest first` gives the N most recent episodes, oldest first. With no sorts, the order is the source order, each source in its natural order (all of A, then B, then C).

An empty source set with a rule applied acts on all episodes from all subscribed feeds.

A rule applied to a populated source set acts only on that set.

## Notes

- The rule system is intentionally analogous to a SQL SELECT: sources = FROM, filters = WHERE, sorts = ORDER BY
- Each new rule type can be added incrementally without changing the underlying data model

## Open questions

- Specific episodes can be added or included by hand
  ([[US-0202 Add episodes to playlists]], [[US-0216 Include episodes in playlist]]). How
  that relates to rules is decided in
  [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]:
  a hand-picked source owned by the playlist, not MVP. Filters apply to hand-picked
  episodes as normal.

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
