# Authenticated Feed Subscriptions
---
priority: low
depends: subscription
---

Some podcast feeds require authentication to access (e.g. Patreon supporter
feeds, supporting-cast, private RSS feeds). This feature extends the base
subscription model to support these.

## Known Requirements

- Store credentials or tokens securely (per-subscription)
- Pass credentials when fetching feed URI and downloading episode files
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
