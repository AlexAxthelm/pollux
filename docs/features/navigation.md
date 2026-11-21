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

- **Where is Settings reached from?** Decided by the maintainer: global settings are reached
  from a gear at the top right of the library screen; other settings pages are reached contextually and added as they are built. A
  subscription's settings come from a gear icon on its page, and open a page clearly marked
  as editing that subscription. This
  supersedes the earlier "contextual only" wording below, kept as written: this note said
  "accessible from contextual menus throughout the app, not a top-level destination";
  [[settings]] said "accessible from a gear/settings icon in the library toolbar, or via
  contextual entry points".

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
