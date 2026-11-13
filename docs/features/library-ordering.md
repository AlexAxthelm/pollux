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
it rather than after everything else.

**Current behavior** is naive code-point ordering, not locale-aware collation. The
core sorts with `title.to_lowercase()`, which folds `Ä` to U+00E4, past ASCII `z` at
U+007A. For a mixed list the measured output is `["apple", "Banana", "eagle",
"Zebra", "Ärger", "Ångström", "Émile"]`. That matches no real locale: German sorts
`Ä` with `A`, Swedish places it after `Z` but ordered `Å < Ä < Ö`, and we emit
`Ärger` before `Ångström`.

A real fix means locale-aware collation (e.g. the `icu_collator` crate) plus a
decision about where the locale comes from. That is a dependency and architecture
call, so per [[CONTRIBUTING]] it wants discussion before implementation rather than
being folded into a feature PR.

This is deliberately **not** covered by a test: asserting the current order would pin
behavior no locale actually wants, and would make a correct implementation later look
like a regression.

Whatever collation lands here should also apply to the later ordering options above.

## Decisions

- [[ADR-0024 The order the library shows is decided by the core]]

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
