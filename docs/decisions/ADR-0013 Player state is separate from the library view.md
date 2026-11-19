---
type: decision
status: accepted
feature:
  - "[[player-performance]]"
related:
  - "[[US-0162 Minimize playback resource usage]]"
---
# Player state is separate from the library view

## Context

While playing, the engine reports the position about once a second and the core
renders on each report, so the shell re-reads the whole view model every second.
With a long episode list open, that woke the list every second. Simulator
(debug-build) numbers suggested about 4 to 5% CPU.

## Decision

The player state lives in its own object (`PlayerState`), and `Core` only
republishes `view` (what the library and the episode list read) when something
other than the player changed. A tick therefore wakes only the mini-player and
the full player, not the episode list. Covered by `CoreWiringTests` ("The player
changes without disturbing the list").

The core keeps rendering on every tick; further reduction is not needed.
Measured on an iPhone 14 Pro Max (release core and app, a 592-episode feed,
Time Profiler, 30 s each): paused with the list open 0.03% CPU; playing on the
library screen 0.13%; playing with the list open 0.12%.

## Consequences

- The remaining options (stop rendering on ticks, shrink the payload, stop
  recomputing previews) are kept in [[player-performance]] in case this changes,
  for example with a much larger feed.
- Scrolling a long list is the cost worth a look, in the library work.
