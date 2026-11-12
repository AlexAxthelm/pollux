---
type: feature
status: draft
priority: MVP
area: player
parent: "[[player]]"
depends:
  - "[[player]]"
  - "[[downloads-queue]]"
  - "[[episode]]"
---
# Streaming and local playback

An episode the listener doesn't have yet should start playing at once, and end up
downloaded. This file covers how playback moves between the stream and the local
file, and what happens when either goes wrong. How downloads are queued and
stored is the [[downloads]] epic.

## Behavior

- Pressing play on an episode that isn't downloaded **streams** it and **queues a
  download**. When the download finishes, playback moves onto the local file at
  the current position.
- If the engine can't use the downloaded file, playback falls back to the stream
  and the file is replaced once. If the replacement is unusable too, the file is
  marked failed and playback carries on from the stream.
- Playback never downloads against the listener's wishes: a download they
  cancelled or deleted is not brought back by playing.
- A stream that fails should be retried with the episode's up-to-date address.
- While a stream waits for data, the lock-screen position stops and the app says
  so.

The full reasoning and the exact rules are in the decisions below.

## Decisions

- [[ADR-0003 Streaming starts at once and swaps to the download]]
- [[ADR-0004 Playback never downloads against the listener's wishes]]
- [[ADR-0001 Engine events carry a session id]]

## Known limitations and follow-ups

### Audible stream-to-local swap

When a download finishes while its episode is streaming, playback moves onto the
local file with a `Load` at the playhead. That is audible: the load itself leaves a
brief gap, and because the core's position is the last whole-second tick, up to
about a second of audio repeats. (User story: [[US-0156 No gap when switching from streaming to local]].)

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

### A refresh reaches the active episode only when its feed is open

The player keeps its own copy of its episode. When a refresh renews or corrects the audio
URL, that copy is updated when the open feed's episode list reloads
(`refresh_active_from_list`), so retrying a failed stream loads the new address. If a
different feed is open, the copy keeps the old URL until relaunch, and a retry from the
mini-player loads the old address. Fix: update the active episode when a refresh is
stored, whichever feed is open. (User story: [[US-0157 Renew links to streams]].)

### Deleting the active episode's download does not switch playback to the stream

If the file of the episode playing from local is deleted mid-play, the engine keeps reading
the already-open file, so playback continues (an offline seek ten minutes ahead still
played), and the stream fallback only happens on the next real load. That is harmless for
the listener, though the space is not freed until the item is released. The delete is
remembered, so the fallback streams without downloading it again, but only for the session:
after a relaunch, playing the episode re-queues the download (see "Declined downloads"
below). Open question: reload from the stream as soon as the active episode's download is
deleted? Doing so would cost a brief gap, to replace audio that is still playing fine.

### Declined downloads and the bad-file memory last for the session

Cancelling or deleting an episode's download is remembered so playback doesn't bring it
back (`declined_downloads`), and a file the engine couldn't use is remembered so it isn't
downloaded again and again (`ActivePlayback::local_unusable`). Both are kept for the
session only. After a relaunch the download-and-fail cycle is still bounded (a file that
proves bad twice is marked failed) but can run a couple of times, and a declined download
may start again. Persisting them needs a column or table. (User story: [[US-0159 Remember cancelled downloads]].)

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
the loop stays closed. A local file that loads clears it, as now. (User story: [[US-0160 Retry retry-able downloads]].)

### The lock-screen clock keeps running while a stream stalls

When a stream stalls (the engine waits for data), playback is still wanted but the audio is
not advancing, and the engine reports no position ticks. Nothing tells the core, which still
says "playing", so the lock-screen card keeps its rate of 1 and the system runs its clock on:
the playhead creeps forward through audio that has not been heard, then jumps back when the
stream recovers. The card's drift check does not catch it, because nothing re-renders in the
meantime. Reported in review (Copilot and Codex).

Fix, when it matters, in four parts:

1. The engine watches whether the player is waiting for data (`timeControlStatus` is
   `waitingToPlayAtSpecifiedRate`) and reports when that starts and stops, with the load's
   session, only on a change. A new load starts afresh (the engine forgets what it reported
   and the core forgets the stall).
2. The core records it apart from play/pause intent (`is_buffering`, a `PlayerBuffering`
   event): the listener has not paused, so `is_playing` must not change. The view shows it
   only while playing.
3. The lock-screen card's rate is 0 while stalled (1 only when playing and not stalled), the
   card is rewritten as soon as the stall starts or ends, and the drift check does not
   predict the playhead advancing during a stall.
4. A test that drives the real core with a stall and no download or refresh event, and checks
   the card's rate goes 1, then 0, then 1. (A test of the card's policy alone is not enough:
   it supplies the update that this path never produces.)

The app's own controls do not show a stall either; a spinner on the play button would be the
natural companion. (User story: [[US-0165 Buffering indicator]].)

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
