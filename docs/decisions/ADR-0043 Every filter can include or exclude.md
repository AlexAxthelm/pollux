---
type: decision
status: accepted
feature:
  - "[[playlists-rules]]"
  - "[[playlists-editor]]"
related:
  - "[[US-0215 Playlist episode exclusion]]"
  - "[[US-0225 Playlist filter for status]]"
  - "[[ADR-0027 A playlist is a query over sources]]"
---
# Every filter can include or exclude

## Context

The editor spec puts an include/exclude toggle on every filter, and `DATA_MODEL` gives `PlaylistFilter` an include/exclude polarity. The filter tables in [[playlists-rules]] had neither. Several stories ask to include or exclude by status, date, duration, stars and title.

## Decision

Every filter carries an include/exclude polarity. The user is responsible for sensible combinations. Filters apply top to bottom, and order matters.

## Consequences

- MVP filters are "include/exclude playback status" and "include/exclude most recent / oldest N".
- The polarity is part of the stored filter from the start, so exclusion stories need no model change.
- Deduplicate ([[playlists-rules]]) is a filter with no criteria. It is on or off and has no include/exclude toggle.
