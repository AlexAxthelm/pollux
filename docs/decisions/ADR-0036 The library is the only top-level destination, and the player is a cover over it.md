---
type: decision
status: accepted
feature:
  - "[[navigation]]"
related:
  - "[[BUG-0001 Mini-player covers the Feed URL field]]"
---
# The library is the only top-level destination, and the player is a cover over it

## Context

Pollux has one main screen, and the player has to be reachable from anywhere in the app
without becoming a destination of its own.

## Decision

- The **library** is the root of a single navigation stack. Subscription details and
  episode details are pushed onto it.
- The **full player** is a full-screen cover from the root, not a pushed screen. Tapping
  "From: …" dismisses the cover and shows the podcast.
- The **mini-player** is a bottom inset on the root, so it shows on every screen.
- **Settings** and **Downloads** are not top-level destinations; see [[navigation]].

## Consequences

- Because the mini-player is an inset on the root, it can sit over content at the bottom of
  a screen. [[BUG-0001 Mini-player covers the Feed URL field]] is one case of this.
