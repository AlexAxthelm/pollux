---
type: feature
status: draft
priority: mid
area: storage
parent: "[[storage]]"
depends:
  - "[[storage-limits]]"
  - "[[storage-protection]]"
  - "[[storage-notifications]]"
  - "[[episode]]"
---
# Storage eviction and retention

Rules for which downloaded episodes Pollux keeps and which it lets go of. This is
a *future* capability: until it exists, a full store simply stops downloading
(see "Fail on full" in [[storage-limits]]).

## Default priority order

When automatic eviction is implemented, the proposed priority order (lowest to
highest protection) is:

1. **Evict first**: played episodes, no flag, oldest first
2. **Evict next**: unplayed episodes, oldest first
3. **Never evict**: flagged / starred episodes (see [[storage-protection]])

All eviction rules should be user-configurable for power users.

## Rules

- **Storage-based rules** — evict or retain episodes based on how full storage
  is, so the user doesn't have to manage storage by hand.
- **Property-based rules** — evict or retain based on episode properties: date,
  size, listened status.
- **Auto-delete** — e.g. an unlistened episode deletes itself when a newer one
  is released, so stale episodes don't pile up.
- **Playlist retention** — a playlist can carry its own retention rules, so some
  playlists are always kept ready to listen.

### Action mode

Because Pollux should never silently delete content the user might care about,
each rule has an action mode:

- **Suggest** (default) — matching episodes are flagged as "this could be
  deleted" and surfaced to the user, who confirms. See
  [[storage-notifications]].
- **Auto** (opt-in) — matching episodes are deleted without asking.

### Interactions between rules

Rules can conflict, especially when they inherit from multiple sources (several
playlists each saying to retain or evict the same episode). The user should be
able to control how rules interact.

**Default: retain.** Conflicting retentions resolve in the safe direction, so a
file is not lost by accident.

## Open questions

- Which rule types ship first, and which are left for power users?
- How are rule conflicts shown to the user so the outcome isn't surprising?
- Should presets from [[storage-styles]] be expressed as bundles of these rules?

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
