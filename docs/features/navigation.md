---
type: feature
status: draft
priority: MVP
area: navigation
depends:
  - "[[library]]"
  - "[[player]]"
  - "[[mini-player]]"
---
# Navigation Structure

Pollux has a minimal top-level navigation model:

- **Library** (home) — always accessible
- **Now Playing** — accessible via mini-player tap when something is active
- **Settings** — accessible from contextual menus throughout the app, not a
  top-level destination
- **Downloads** — accessible from the library view, not a top-level destination

## As built

- The library is the root of a single navigation stack. Subscription details and episode
  details are pushed onto it.
- The full player is a full-screen cover from the root, and the mini-player is a bottom
  inset on the root. See [[ADR-0036 The library is the only top-level destination, and the player is a cover over it]].
- **Not built yet:** there is no Settings screen and no Downloads screen.

## Open questions

- #needs-decision **Where is Settings reached from?** This note says "accessible from
  contextual menus throughout the app, not a top-level destination". [[settings]] says
  "accessible from a gear/settings icon in the library toolbar, or via contextual entry
  points". The library toolbar in [[library]] lists no Settings button. All three are kept
  as written.

## Decisions

- [[ADR-0036 The library is the only top-level destination, and the player is a cover over it]]

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
