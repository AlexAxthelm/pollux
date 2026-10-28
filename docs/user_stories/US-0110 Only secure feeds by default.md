---
type: user-story
status: draft
feature:
  - "[[subscription]]"
related:
  - "[[US-0107 Allow http (unsecure)]]"
  - "[[US-0111 Scope security exceptions]]"
---
As a subscriber
I want the default to only fetch secure feeds, but with a per-feed override
so that I can expect security, and opt in to insecure feeds

**Unresolved:** the per-feed override story is in tension with how ATS works.
  `NSExceptionDomains` entries are static Info.plist values fixed at build time,
  and there is no runtime API to add a domain — but subscription URLs are supplied
  by the user long after we ship, so the hosts needing an exception are unknowable
  in advance. A genuinely per-feed opt-in therefore cannot be expressed through
  ATS scoping. The realistic implementation is the inverse: declare
  `NSAllowsArbitraryLoads` and enforce the https-only default in our own code,
  with the per-feed opt-in as our gate rather than the OS's. Same behavior for the
  user, but the OS stops being the backstop and a bug in our gating fails open —
  so this needs a deliberate decision, and the maintainer story above should be
  rewritten once it's made.