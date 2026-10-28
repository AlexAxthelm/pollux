---
type: user-story
status: draft
feature:
  - "[[subscription]]"
related: []
---
As a subscriber
I want to subscribe to shows whose feeds are still http-only
so that older or self-hosted shows aren't unreachable

Notes from current state:
* Entering an `http://` feed URL surfaced Apple's raw ATS string verbatim:
  "The resource could not be loaded because the App Transport Security policy
  requires the use of a secure connection." That is accurate but not actionable,
  and it is Apple's wording rather than ours.
* `iOS/Pollux/Info.plist` currently declares no ATS exceptions, so iOS default
  behavior (arbitrary loads disabled) applies.