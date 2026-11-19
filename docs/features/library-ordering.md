---
type: feature
status: draft
priority: MVP
area: library
parent: "[[library]]"
depends:
  - "[[library]]"
  - "[[settings]]"
---
# Library ordering

How the Playlists and Shows sections of the [[library]] are ordered.

## Default

Default ordering for both Playlists and Shows sections is **alphabetical**.

## Later

Additional ordering options: by last updated, custom (drag to reorder). Manual reordering
is behind a mode toggle (e.g. three-dots menu → "Reorder" or "Change Order"), not inline drag by
default.

## Consistency

The ordering is the same wherever the library is listed, so a show doesn't move
depending on which screen the user came from.

## Locale-aware sorting (unresolved)

A listener whose shows aren't all in English should see the library sorted the way
their language sorts, so a show starting with "Ä" or "É" appears where they expect
it rather than after everything else. Today it is naive code-point ordering. The
findings and the open decision are in
[[ADR-0025 Library sorting uses locale-aware collation]].

## Decisions

- [[ADR-0024 The order the library shows is decided by the core]]
- [[ADR-0025 Library sorting uses locale-aware collation]] (draft)

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
