# Audio Playback
---
priority: MVP
depends: episode, mini-player, subscription, settings
---

The player is the primary listening interface. It integrates with the OS media
system where available, and surfaces controls for the currently active episode
and playlist.

## OS Integration

- Use system media intents for playback control (lock screen, headphone buttons,
  car/speaker integration)
- Resume position on return: if the user leaves and comes back, playback resumes
  approximately where they left off. A small rewind on resume (e.g. 3-5s) is
  acceptable and common practice for context overlap. User can disable that
  behavior (see `settings.md`)

## Paged Content Area

The top portion of the player is a paged view (swipeable carousel with dot
indicators). MVP pages:

1. **Artwork**: episode image → feed image → placeholder. Chapter art shown
   if available and episode is in a chapter (see `episode-chapters.md`).
2. **Show notes**: the episode's show notes, rendered as rich text (the same
   renderer as the episode detail page) and scrollable. The page, and the dot
   indicators, are left out when the episode has no show notes. Timestamp
   detection and tappable timestamps are not built yet; see
   `player-show-notes.md`.

*(Future)* Additional pages: chapter list, visualizer (see `visualizer.md`)

The artwork fills the width of the player (as large a square as fits). A
user-configurable player layout is a possible later feature; the default layout
below is the only one for now.

Primary controls (play/pause, skip) are drawn in the theme's accent color; the
secondary row (hide, output routing, options) uses the text color.

Note: for large screens, MVP will remain paged, but multiple pages may be shown
simultaneously if space allows.

## Scrubber / Position

A draggable progress bar showing current position in the episode.

- Current time shown on left, total time on right
- Tapping the total time toggles to remaining time
- *(Future)* Chapter markers overlaid on the scrubber

## Controls

### Skip Buttons

Forward and back skip buttons, configurable independently:

- **Default**: 30s forward, 15s back
- **Configurable**: global default, overridable per feed (cascades global → feed)
  (see `settings.md`)
- Button label displays the current increment (e.g. "+30" / "-15")
- Skipping past the end or beginning of the current file does not overshoot
  into the next/previous episode

### Chapter Navigation

Chapter back / chapter forward controls appear when the episode has chapters.
See `episode-chapters.md` for full behavior.

### Play / Pause

Standard toggle. Shows pause symbol when playing, play symbol when paused.

## Source Context

The player displays the active playlist/source context (e.g. "From: Music -
Boppy"). This is tappable and navigates back to the source playlist or
subscription view.

This also appears in the mini-player (see `mini-player.md`).

## Player Options Menu

Accessible from the player UI. MVP contents:
- OS output/routing options (where available)
- *(placeholder: speed control — see `speed-control.md`)*
- *(placeholder: sleep timer — see `sleep-timer.md`)*
- *(placeholder: equalizer — see `equalizer.md`)*

## Episode Options Menu

Standard three-dots episode context menu (see `episode.md`). Available from
the player without leaving the view.

## Default Layout

```
NavBar
[ Paged content area: Art | Show notes (if any) ]
[ Dot indicators ]
Position / scrubber
(Chapter Back) | Title & Chapter, From Source | (Chapter Next)   ← if chapters present
(-15s)         |   Play/Pause    | (+30s)
(Hide)         | [Player options]| (Episode options ...)
```

"Hide" collapses to mini-player and is equivalent to navigating back up the
nav stack.

## Active Playlist Persistence

The active playlist and current position are restored on cold start, so the
mini-player is visible immediately if something was playing when the app closed.

## Decisions (MVP implementation)

Recorded so they are not silently reversed. The core owns all of this policy
(`shared/src/player.rs`); the shell's `PlaybackManager` only drives the audio
engine.

- **Streaming.** Playing an episode that isn't downloaded streams from its
  enclosure URL *and* queues a download. When the download finishes, playback
  moves onto the local file at the current position (a `Load` at the playhead).
  If the engine reports the downloaded file unusable (missing, unreadable or
  undecodable), playback falls back to streaming and the episode is reset to
  not-downloaded and re-queued. Any other failure (another app holding the audio
  session, playback stopping mid-play) leaves the download alone, pauses with an
  error, and a later play retries the same file.
- **Played tolerance.** Played is decided when the listener *leaves* an episode,
  never as a side effect of saving progress. It happens when the engine reaches
  the end, when they pause, or when they start another episode, while within
  **15s** of the end (`PLAYED_TOLERANCE_SECS`, but never more than a quarter of
  a short episode). The status becomes played, the stored position resets to 0,
  the saved context is cleared and the player goes inactive. The engine's
  duration wins over the feed's; with no known duration only the engine reaching
  the end finishes an episode. *Will become a user setting.*
- **Not leaving.** Periodic saves and backgrounding only record in-progress and
  a position, so listening on through the last seconds with the screen locked is
  never cut short. A system pause (a call, Siri, unplugged headphones) keeps the
  episode and its place and never counts as finishing, so it can resume and play
  out the end. When an audio-session interruption (a call, Siri) ends, playback
  resumes only if that interruption is what paused it and the system says it may:
  the system also says "may resume" after interruptions that found playback
  already paused, and the listener's pause then stands. A pause from unplugged
  headphones is never resumed automatically, and playing or scrubbing by hand
  during an interruption cancels the auto-resume.
- **Source.** Playback remembers what it was started from as an `EpisodeSource`
  (a subscription today; see `DATA_MODEL.md`), saved with the play context and
  restored with it. The "From:" row names it and tapping it navigates back to it.
  It is separate from the episode's own feed, which is what the lock screen shows
  as the artist. Playback started from a feed's episode list uses that feed; once
  playlists exist the view that starts playback will say which source it is.
- **Restore.** An episode already marked played is not restored at launch (its
  saved context is cleared); an in-progress one comes back paused at its place.
- **Sessions.** Each load into the engine gets a session id, and everything the
  engine reports carries it. The core ignores news from any other session, so a
  late tick, end or failure from a replaced item (a source swap, a retry, another
  episode) can't affect the current one.
- **Position persistence.** Written at most every 10s while playing, and on
  pause, seek, backgrounding, switching episode, and end. Per-second ticks are
  transient.
- **Resume rewind.** 3s on any resume after a pause (and on cold-start restore
  and when resuming a saved position), clamped at 0. Hard-coded until Settings
  exists (`defaults.rs`), as are the 30s/15s skips.
- **No auto-advance (yet).** At the end of an episode it is marked played and the
  player goes inactive (the mini-player disappears). "Next episode" arrives with
  playlists.
- **Presentation.** The full player is a full-screen cover from the root; the
  mini-player is a bottom inset on the root. Tapping "From: …" dismisses the
  cover and shows the podcast.
- **OS integration.** Audio session `.playback` / `spokenAudio`, background
  audio mode, pause on headphone unplug, pause on interruption (auto-resume only
  if the system says so). Lock-screen commands: play, pause, toggle, skip
  ±(30/15), and scrubbing. Next/previous track are disabled (skip buttons stand
  in for them, per podcast convention).
- **Errors.** A playback failure leaves playback paused with a transient banner
  in the player and mini-player; pressing play retries.

## Known limitations and follow-ups

### Audible stream-to-local swap

When a download finishes while its episode is streaming, playback moves onto the
local file with a `Load` at the playhead. That is audible: the load itself leaves a
brief gap, and because the core's position is the last whole-second tick, up to
about a second of audio repeats. (User story: `user_stories/player/playback.md`,
"carry on seamlessly when it finishes downloading".)

Options, cheapest first:

1. **Continue from the engine's exact time.** A `Swap` operation meaning "carry on
   from wherever you are", instead of a `Load` at the last tick. This removes the
   repeated second. Prepare (preroll) the local item before replacing the current
   one, so the load gap shrinks too.
2. **Cue and swap at a point just ahead.** On download completion the core sends a
   `Cue` for the local file about 2 to 3 seconds ahead. The shell prepares the local
   item, seeked to that point, while the stream keeps playing, then hands over there
   (an `AVQueuePlayer`, ending the stream item with `forwardPlaybackEndTime`). The
   lead is only preparation time: the hand-over is meant to be seamless, so it needs
   no silence detection and no chapters. If the item isn't ready in time, fall back
   to an immediate swap.
3. **Swap at a natural break.** A pause or a seek is already a discontinuity, so if
   either arrives while a cue is pending the shell swaps then, which is inaudible.

Notes for whoever builds this:

- The session scheme already supports it: the core adopts the new session when it
  cues, and the stream item's late end-of-file event carries the old session and is
  ignored.
- Option 2 means converting the engine from `AVPlayer` to `AVQueuePlayer`, which
  touches seeking, ticks and end-of-file handling. Budget for that.
- Whether it is audibly gapless can only be judged by listening on a device (MP3
  decoder priming can leave a small gap even when the hand-over is gapless on paper).
  The simulator can confirm the playhead continues through the swap, not how it sounds.
- Hosts that insert ads dynamically can serve different audio per request, so the
  download may not match what was streamed. At a given timestamp the swap would then
  land in different content. No swap strategy fixes that; it applies to today's swap
  too. Worth knowing before judging a swap by ear on such a feed.

### Cost of the once-a-second render (needs a device measurement)

While playing, the engine reports the position about once a second and the core renders
on each report, so the shell re-reads the whole view model every second. Nothing is wrong
with that in principle, but how much it costs on a real device has **not been measured**,
and the simulator numbers below are from debug builds, which overstate it.

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
