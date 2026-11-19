---
type: feature
status: accepted
priority: mid
area: library
parent: "[[library]]"
depends:
  - "[[library]]"
---
# Library scroll performance

A long episode list should scroll smoothly without heating the phone or draining
the battery, so the listener can browse a show with hundreds of episodes freely.

## Current state

Measured on an iPhone 14 Pro Max (release build, a 592-episode feed): continuous
flick scrolling cost about **42% CPU, 28% of it on the main thread**. Playback itself
is not the cost (0.1% while playing with the list open); see
[[player-performance]] and
[[ADR-0013 Player state is separate from the library view]].

## Likely cause

Artwork refetching on scroll is the likely contributor (issue #14). This is a
suspicion, not a finding: **profile before changing anything.**

## Open questions

- How much of the cost is artwork refetching, and how much is the row views
  themselves?
- The episode list ships every episode's raw HTML description even though rows only
  need the preview; shipping it on demand would cut the payload. That changes how the
  episode list is projected, so it belongs here (see "Shrink the payload" in
  [[player-performance]]).

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
