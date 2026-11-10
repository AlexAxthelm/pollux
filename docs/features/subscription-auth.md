---
type: feature
status: draft
priority: low
area: subscription
parent: "[[subscription]]"
depends:
  - "[[subscription]]"
---
# Authenticated Feed Subscriptions

Some podcast feeds require authentication to access (e.g. Patreon supporter
feeds, supporting-cast, private RSS feeds). This feature extends the base
subscription model to support these.

## Known Requirements

- Store credentials or tokens securely (per-subscription)
- Pass credentials when fetching feed URI and downloading episode files, so
  subscribers can download episodes from access-controlled feeds (see
  [[downloads]])
- UI for entering / updating credentials within subscription settings
- Handle auth failures gracefully (notify user, don't silently fail)

## Known Services

- Patreon (private RSS feed with token in URL)
- Supporting Cast
- Supercast
- Generic HTTP Basic Auth

## Open Questions

- Should credentials be synced across devices (see `sync.md`)? Requires
  secure handling.
- How should token rotation / expiry be handled?
- Are OAuth flows needed, or is token-in-URL / Basic Auth sufficient for
  known services?

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
