---
type: user-story
status: draft
feature:
  - "[[subscription-transport-security]]"
related:
  - "[[US-0107 Allow http (unsecure)]]"
  - "[[US-0111 Scope security exceptions]]"
---
As a subscriber
I want the default to only fetch secure feeds, but with a per-feed override
so that I can expect security, and opt in to insecure feeds

**Unresolved:** the per-feed override is in tension with how ATS works. See
[[ADR-0026 Feed transport security is enforced by the app, not scoped through ATS]].
