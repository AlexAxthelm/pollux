---
type: decision
status: draft
feature:
  - "[[subscription-transport-security]]"
related:
  - "[[US-0107 Allow http (unsecure)]]"
  - "[[US-0110 Only secure feeds by default]]"
  - "[[US-0111 Scope security Exceptions]]"
---
# Feed transport security is enforced by the app, not scoped through ATS

## Context

Some shows are still served over plain http. The wanted behavior is secure by
default, with a per-feed override so the user can opt in to an insecure feed, and any
exception scoped to the feeds that need it
([[US-0110 Only secure feeds by default]], [[US-0111 Scope security Exceptions]]).

Today `iOS/Pollux/Info.plist` declares no ATS exceptions, so iOS default behavior
(arbitrary loads disabled) applies, and an `http://` feed is unreachable. Entering one
surfaces Apple's raw ATS string verbatim.

The per-feed override is in tension with how ATS works. `NSExceptionDomains` entries
are static Info.plist values fixed at build time, and there is no runtime API to add a
domain. Subscription URLs are supplied by the user long after we ship, so the hosts
needing an exception are unknowable in advance. A genuinely per-feed opt-in therefore
cannot be expressed through ATS scoping.

## Decision

**Not yet decided.** The realistic implementation is the inverse of the current
setup: declare `NSAllowsArbitraryLoads` and enforce the https-only default in our own
code, with the per-feed opt-in as our gate rather than the OS's.

- **For:** same behavior for the user, and the only way to offer a real per-feed
  opt-in.
- **Against:** the OS stops being the backstop, and a bug in our gating fails open.

This needs a deliberate decision.

## Consequences

- The maintainer story about scoping exceptions ([[US-0111 Scope security Exceptions]])
  should be rewritten once it is made.
- The related behavior (try https first, an informative insecure-feed prompt) is in
  [[subscription-transport-security]].
