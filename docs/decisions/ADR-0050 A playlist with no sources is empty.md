---
type: decision
status: accepted
feature:
  - "[[playlists]]"
  - "[[playlists-rules]]"
  - "[[playlists-editor]]"
related:
  - "[[ADR-0027 A playlist is a query over sources]]"
  - "[[ADR-0028 A subscription is an implicit single-feed playlist]]"
---
# A playlist with no sources is empty

## Context

[[ADR-0027 A playlist is a query over sources]] said that an empty source set, with a rule applied, acts on all episodes from all subscribed feeds. With Deduplicate on by default that makes every new playlist show everything, and a playlist whose last source is removed (or whose hand-picked episodes are all dropped on unsubscribe) quietly becomes "everything".

## Decision

A playlist with no sources is an empty list. To use every show, the user adds them as sources. The list of shows in the editor has an **Add all shows** button at the top.

## Consequences

- This replaces the "empty source set" rule in ADR-0027. The library and the "All subscriptions" convenience playlist stay as they are ([[ADR-0028 A subscription is an implicit single-feed playlist]]).
- #needs-decision **What does "Add all shows" add?** It could add every current show as its own source row, so shows subscribed later are not included. Or it could add one "All shows" source that always follows the library. Not decided.
