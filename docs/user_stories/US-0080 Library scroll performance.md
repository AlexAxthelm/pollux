---
type: user-story
status: draft
feature:
  - "[[library]]"
related: []
---
As a listener
I want a long episode list to scroll smoothly without heating the phone or draining
the battery
so that I can browse a show with hundreds of episodes freely

notes from current state:
* Measured on an iPhone 14 Pro Max (release build, a 592-episode feed): continuous
  flick scrolling cost about 42% CPU, 28% of it on the main thread. Playback itself is
  not the cost (0.1% while playing with the list open). Artwork refetching on scroll is
  the likely contributor (issue #14); profile before changing anything.