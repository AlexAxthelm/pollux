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
| First N episodes | N (count) | Keeps the first N episodes of the list as it stands at this point. "The N most recent" is a sort by age, newest first, then First N episodes. Not the same as the episode limit, the per-playlist safety cap ([[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]]). |
| N per source | N (count), direction (newest / oldest) | Keeps the N most recent or the N oldest episodes of each source in the Sources section, by release date. "Most recent 1" per source is one episode from each show. Each source counts separately; an included playlist is one source, and the All shows source counts as one source for each show ([[ADR-0050 A playlist with no sources is empty]]). |
| Playback status | unplayed / in-progress / played (multi-select) | Enables "in progress" playlists, unplayed-only queues, etc. In-progress means not played, with the playback position not at the start. |
| Deduplicate | none | Keeps the first occurrence of each episode at this point in the rules. On by default, at the top of the filters; it can be moved or turned off. It has no include/exclude toggle. A playlist has at most one, and it can be added again at the top after removal. See [[US-0217 Playlist deduplication]], [[US-0218 Playlist allows duplicates]]. |
| Title | text | Includes or excludes episodes whose title contains the given text (a simple match). Not in the original MVP scope, but cheap to include. A general regular-expression filter may follow. See [[US-0224 Playlist filter on titles]]. |

### Later

| Rule | Parameters | Notes |
|---|---|---|
| Date range | start / end date | "Episodes released after X" |
| Duration | min / max seconds | Short-form only, long-form only, etc. |
| Starred / favorited | — | Only episodes the user has flagged |
| Tag | tag name(s) | Depends on tagging feature |
| Download status | downloaded / not downloaded | Needed for the **Downloaded** convenience playlist ([[playlists-convenience]]). |
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
| Manual ordering | none | Appears when the playlist has pins and puts them at the front or back in hand-set order. See [[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]. |
| By source (interleaved) | strategy: round-robin / proportional / date-normalized | Solves the archive-clumping problem; see [[playlists-archive]] |
| By duration | asc / desc | |
| By play count | asc / desc | Surfaces favorites or finds unlistened gems |
| Random | - | - |

## Rule Composition

Rules are stacked and applied in order. Filters only intersect: each narrows what the ones above left, and there is no OR between filters (add sources for that). The UI should make reordering easy and show a live preview of the resulting episode list.

Filters, sorts and Deduplicate share one stack and can be interleaved ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]). Each rule is applied to the result of the ones above it. For example, `sort: newest first`, `filter: first N episodes`, `sort: oldest first` gives the N most recent episodes, oldest first.

A playlist with no sources is empty ([[ADR-0050 A playlist with no sources is empty]]).

A rule acts on the combined list of the playlist's sources.

## Notes

- The rule system is intentionally analogous to a SQL SELECT: sources = FROM, filters = WHERE, sorts = ORDER BY
- Each new rule type can be added incrementally without changing the underlying data model

## Hand-picked episodes

Specific episodes can be added or included by hand
  ([[US-0202 Add episodes to playlists]], [[US-0216 Include episodes in playlist]]). How
  that relates to rules is decided in
  [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]:
  a hand-picked source owned by the playlist, not MVP. Filters apply to hand-picked
  episodes as normal. Nothing is open here.

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
