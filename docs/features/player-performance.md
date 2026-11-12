---
type: feature
status: draft
priority: mid
area: player
parent: "[[player]]"
depends:
  - "[[player]]"
---
# Playback performance

Playback should be light on battery and CPU, even with a long episode list open,
so the listener can listen all day without noticing it.

The current position of this work: the render cost was measured on a device and
needs no further reduction ([[ADR-0013 Player state is separate from the library view]]).
The investigation is kept below in case that changes.

## Cost of the once-a-second render (measured on a device: no follow-up needed)

While playing, the engine reports the position about once a second and the core renders
on each report, so the shell re-reads the whole view model every second.

**Measured on an iPhone 14 Pro Max** (release core and release app, a 592-episode feed, Time
Profiler, 30 s each, over USB): paused with the list open 0.03% CPU; playing on the library
screen 0.13%; playing with the list open 0.12%. Scrolling the list, as a control to show
the sampler works, was about 42%. The simulator's debug-build numbers below (4.1 to 5.4%)
overstated the cost by more than 30 times. So the render does not need reducing, and the
options below are kept only in case that changes (a much larger feed, say). The Energy Log
was not run. Scrolling a long list is the cost worth a look, in the library work: continuous
flicking of the 592-row list cost about 42% CPU, 28% of it on the main thread (see also
issue #14, artwork refetching on scroll).

The rest of this section is the simulator investigation. The simulator numbers are from
debug builds, which overstate it.

What was done: the player state lives in its own object (`PlayerState`), and `Core` only
republishes `view` (what the library and the episode list read) when something other than
the player changed. A tick therefore wakes only the mini-player and the full player, not
the episode list. Tests: `CoreWiringTests` ("The player changes without disturbing the
list").

What was measured (a 375-episode feed, an M-series Mac, the iPhone simulator):

| What | Cost |
|---|---|
| View model per render | about 235 KB; 150 KB of it is episode descriptions and their previews |
| Build the view in Rust, release | about 0.5 ms (nearly all of it stripping and cloning descriptions) |
| Build the view in Rust, debug | about 6 ms (the simulator builds use the debug core) |
| Serialize | 13 µs release, 0.3 ms debug |
| Decode in Swift (debug test build) | about 2 to 2.5 ms |
| App CPU while playing, list open, before the split | about 5.4% |
| Same, after the split | about 4.1% |
| Same, experiment where ticks do not render at all | about 1.0% |
| App CPU while playing on the library screen (before the split) | about 2.2% |

The split removed the SwiftUI part (the list re-evaluating each second). The remaining gap
to 1.0% is probably the per-tick pipeline itself (build, serialize, decode, compare) running
in debug builds. That is an inference, not a measurement.

Follow-up, in order:

1. **Measure a release build on a device** (Instruments: Time Profiler and Energy Log)
   while playing with the episode list open, and compare against paused. If it is small,
   stop here. Note that `make ios-xcodebuild` builds the Rust core without `--release`
   (the Makefile's `package` target runs `cargo swift package` with no profile flag), so
   simulator numbers are not representative. A release option in the Makefile would make
   this repeatable.
2. **If it still matters, stop rendering on ticks.** The core would update its position
   without a render (rendering only on checkpoints and state changes), and the shell would
   drive the displayed position from its own clock fed by the engine. In the experiment this
   reached the 1.0% floor. The cost is a second source of truth for position that has to be
   reconciled on every seek, skip, pause and source swap.
3. **Shrink the payload and stop recomputing previews.** Two separate, smaller wins:
   - The Rust time per render is almost entirely the descriptions: the 200-character previews
     are rebuilt from the raw HTML for every episode on every render (leaving descriptions
     out of the view took it from about 0.5 ms to 29 µs in the measurement, so the exact
     split between stripping and cloning is not known). Computing each preview once, when
     the episodes load, and keeping it with the episode removes that cost from every render,
     not just ticks. This is a core-only change.
   - The list ships every episode's raw HTML description as well, though the rows only need
     the preview and the detail page needs the raw HTML for one episode. Shipping it on
     demand would cut the payload by about 35% (about 65% with the previews gone too). It
     changes how the episode list is projected, which belongs to the library work, so
     coordinate with that.

To re-measure CPU: start playback, open the subscription page with the long list, find the
app's process id with `pgrep -f "Pollux.app/Pollux"`, then `top -l 5 -s 3 -pid <pid> -stats
pid,cpu`. Compare with playback on the library screen and with playback paused. For the Rust
side, time `Pollux::view` and the bincode serialization for the same feed in `cargo test
--release` against plain `cargo test`.

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
