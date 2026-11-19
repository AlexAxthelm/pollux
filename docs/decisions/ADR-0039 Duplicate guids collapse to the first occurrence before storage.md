---
type: decision
status: accepted
feature:
  - "[[feed-parsing]]"
related: []
---
# Duplicate guids collapse to the first occurrence before storage

## Context

Feeds sometimes repeat a `<guid>` across entries. Storage
resolves this by overwriting, and because feeds are conventionally newest-first,
an unguarded overwrite let an *older* duplicate replace a *newer* episode.

(Moved from [[DATA_MODEL]].)

## Decision

`parse_feed` now collapses duplicates before storage, keeping the **first**
occurrence. That choice is deliberate — record it here so it is not silently
reversed.

## Consequences

- The first occurrence wins because feeds are conventionally newest-first. A feed that lists its
  episodes oldest-first would keep the older entry.
- See [[ADR-0038 A missing guid is replaced by an id derived from the enclosure URL and title]] for the
  other identity rule.
