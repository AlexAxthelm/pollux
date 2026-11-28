---
type: decision
status: accepted
feature:
  - "[[playlists]]"
  - "[[playlists-rules]]"
  - "[[playlists-editor]]"
related:
  - "[[US-0249 Use all my shows as one source]]"
  - "[[ADR-0027 A playlist is a query over sources]]"
  - "[[ADR-0028 A subscription is an implicit single-feed playlist]]"
---
# A playlist with no sources is empty

## Context

[[ADR-0027 A playlist is a query over sources]] said that an empty source set, with a rule applied, acts on all episodes from all subscribed feeds. With Deduplicate on by default that makes every new playlist show everything, and a playlist whose last source is removed (or whose hand-picked episodes are all dropped on unsubscribe) quietly becomes "everything".

## Decision

A playlist with no sources is an empty list. To use every show, the user adds the convenience **All shows** source, which is the first item in the list of shows in the editor. It is one source that always follows the library: shows subscribed later are included and unsubscribed ones drop out.

## Consequences

- This replaces the "empty source set" rule in ADR-0027. The library and the "All subscriptions" convenience playlist stay as they are ([[ADR-0028 A subscription is an implicit single-feed playlist]]).
- The "All subscriptions" convenience playlist ([[playlists-convenience]]) is a playlist whose only source is All shows.
- For the per-source rules ("N per source" and "by source", [[playlists-rules]]), the All shows source counts as one source for each show, so "most recent 1 per source" gives one episode from every show, as if each show had been added on its own.
