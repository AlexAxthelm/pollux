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
  error, and a later play retries the same file. Downloads started or retried
  for the active episode don't depend on its feed being open (after a cold start
  no feed is): the core works from the active episode. Pressing play on an
  episode that is still streaming with no download queued or underway (never
  started, or failed) queues it, as starting an episode does. A file the engine
  can't use is replaced by a fresh download once; if the fresh file is unusable
  too, the file itself is bad, so the download is marked failed (with the reason
  beside Retry), playback carries on from the stream, and pressing play doesn't
  fetch it again (otherwise stream, download, swap, fail would repeat for as long
  as playback went on). The memory is per playback session and clears when a
  local file loads; after a relaunch the cycle can run a couple of times before
  it stops. The bad file stays on disk until a retry overwrites it.
  Playback never downloads against the listener's wishes: once they cancel or
  delete an episode's download, starting or resuming it, or replacing a file the
  engine lost, streams without fetching it again. Only asking for the download
  (the Download action) lifts that. This is remembered for the session only.
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
  The context is written once when playback starts, so a failed write is
  retried at every checkpoint until storage confirms it; otherwise a relaunch
  would restore the wrong episode, or none. It is tried at most 5 times in all
  (`CONTEXT_SAVE_ATTEMPTS`): a write that can't succeed isn't repeated for as long
  as playback goes on, and the next episode starts afresh. Nothing is shown to the
  listener; the cost of giving up is that a relaunch may not restore this episode.
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
  The audio session is claimed only when audio is about to play: starting or
  resuming does, but a paused load (a source swap, scrubbing a restored episode)
  does not, so it can't cut off music the listener moved to; a paused load also
  pauses the player, because AVPlayer keeps its rate when its item is replaced.
  The session is released when playback stops, so other apps are told they may
  resume. The system often refuses that while the removed item's I/O is still
  winding down, so a refused release is tried again shortly (five tries in all,
  logged if it never works), and claiming the session again cancels a pending try.
  The retry's effect on real iOS is only checked on a device: play an episode to
  the end with music paused in another app, and see that the music resumes.
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

### Cost of the once-a-second render (measured on a device: no follow-up needed)

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

### Lock-screen controls (an earlier report of invisible controls did not reproduce)

An earlier build showed the lock-screen and Control Center controls invisibly on the
author's iPhone, although they worked. On an iPhone 14 Pro Max (iOS 26.6.2) the title,
artist, artwork, 15 and 30 second skip buttons, scrubber and times all render and work,
and the scrubber does not jitter, so neither candidate fix (setting
`MPNowPlayingInfoPropertyMediaType` to audio, and setting the skip `preferredIntervals`
before the first Now Playing update) was applied. If it comes back, those are where to
start. The card is rewritten only when out of date (`NowPlayingPolicy`), so rewriting too
often is not the cause. A restored, paused episode is not on the lock screen until it has
been played once (the audio engine and remote commands are created lazily, so a
background-refresh launch stays metadata-only).

### Device verification

Verified in the simulator: streaming, skip, pause and resume with rewind, restore at launch,
"From:" navigation, show notes, the interruption and route-change mapping (by posting the
system's notifications), and the engine's end, failure and tick behavior.

Verified on an iPhone 14 Pro Max (iOS 26.6.2): lock-screen rendering and controls; background
audio with a long locked playback (the position is right on unlock), and a relaunch restores
the mini-player paused, 3 s back; interruptions (Siri resumes, a call resumes if it was
playing and does not after a manual pause, unplugging headphones pauses and stays paused); the
route picker opens and works; the stream-to-local swap is audible but small and not jarring,
with position and playback continuing; the offline error banner (it stays until the next
successful action, which is accepted) with the failed-download triangle and Retry still on
the row; and the render cost (above).

Not verified: switching outputs through the route picker (Bluetooth or AirPlay), and the
Energy Log. The test host's AVPlayer completes seeks at once, so the timing of a real seek is
not covered by tests.

### Deleting the active episode's download does not switch playback to the stream

If the file of the episode playing from local is deleted mid-play, the engine keeps reading
the already-open file, so playback continues (an offline seek ten minutes ahead still
played), and the stream fallback only happens on the next real load. That is harmless for
the listener, though the space is not freed until the item is released. The delete is
remembered, so the fallback streams without downloading it again, but only for the session:
after a relaunch, playing the episode re-queues the download (see "Declined downloads"
below). Open question: reload from the stream as soon as the active episode's download is
deleted? Doing so would cost a brief gap, to replace audio that is still playing fine.

### A refresh reaches the active episode only when its feed is open

The player keeps its own copy of its episode. When a refresh renews or corrects the audio
URL, that copy is updated when the open feed's episode list reloads
(`refresh_active_from_list`), so retrying a failed stream loads the new address. If a
different feed is open, the copy keeps the old URL until relaunch, and a retry from the
mini-player loads the old address. Fix: update the active episode when a refresh is
stored, whichever feed is open. (User story: `user_stories/player/playback.md`, "retry a
failed stream with the episode's up-to-date address".)

### Declined downloads and the bad-file memory last for the session

Cancelling or deleting an episode's download is remembered so playback doesn't bring it
back (`declined_downloads`), and a file the engine couldn't use is remembered so it isn't
downloaded again and again (`ActivePlayback::local_unusable`). Both are kept for the
session only. After a relaunch the download-and-fail cycle is still bounded (a file that
proves bad twice is marked failed) but can run a couple of times, and a declined download
may start again. Persisting them needs a column or table. (User story:
`user_stories/player/playback.md`, "remember downloads I turned away".)

### A replacement download that failed is not retried on play

When the engine can't use a downloaded file, playback streams instead and fetches a
replacement once; if the replacement is unusable too, the file is marked bad and not
fetched again (otherwise stream, download, swap, fail would repeat for as long as playback
went on). The memory that stops that loop (`ActivePlayback::local_unusable`) is set at the
*first* failure, before the replacement has arrived. So if the replacement download then
fails for an unrelated reason (offline, say), pressing play no longer retries it for the
rest of the session, although nothing has shown the file to be bad. The listener can still
tap Download, and a relaunch clears the memory.

Fix, when it matters: make the memory three states instead of a flag: trusted, suspect
while a replacement is being fetched, and bad only once the replacement is unusable too.
Only bad stops the retries on play, so a replacement that never arrived stays retryable and
the loop stays closed. A local file that loads clears it, as now. (User story:
`user_stories/player/playback.md`, "retry a download that failed".)

### Unsubscribing from the show that is playing

There is no unsubscribe flow yet (see ROADMAP Phase 5). When it is built, the player has to
handle its episode's show disappearing: the saved play context is deleted with the episode
(`ON DELETE CASCADE`), but the in-memory player would carry on, "From:" would navigate to a
subscription that no longer exists, and a context save would fail (it is capped at 5
attempts). The player should stop and clear when its episode's subscription is removed.
(User story: `user_stories/player/playback.md`, "stop cleanly if I unsubscribe".)
