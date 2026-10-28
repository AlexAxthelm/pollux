---
type: user-story
status: draft
feature:
  - "[[subscription]]"
  - "[[feed-parsing]]"
related:
  - "[[US-0107 Allow http (unsecure)]]"
---
As a subscriber
I want the app to retry over https when I paste an http link
so that I'm never asked to allow insecure access for a host that supports https
anyway

Note: [[feed-parsing]] covers permanant redirects, which overlaps with this, but is tracked elsewhere