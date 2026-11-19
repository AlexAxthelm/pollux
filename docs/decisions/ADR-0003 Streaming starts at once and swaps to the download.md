---
type: decision
status: accepted
feature:
  - "[[player-streaming]]"
  - "[[downloads-queue]]"
related:
  - "[[ADR-0004 Playback never downloads against the listener's wishes]]"
  - "[[US-0131 Play missing episode]]"
  - "[[US-0156 No gap when switching from streaming to local]]"
---
# Streaming starts at once and swaps to the download

## Context

A listener can press play on an episode that isn't downloaded. They shouldn't
wait for a download, but the episode should end up local for offline use.

## Decision

Playing an episode that isn't downloaded streams from its enclosure URL *and*
queues a download. When the download finishes, playback moves onto the local file
at the current position (a `Load` at the playhead).

Failure handling:

- If the engine reports the downloaded file unusable (missing, unreadable or
  undecodable), playback falls back to streaming and the episode is reset to
  not-downloaded and re-queued.
- Any other failure (another app holding the audio session, playback stopping
  mid-play) leaves the download alone, pauses with an error, and a later play
  retries the same file.
- Downloads started or retried for the active episode don't depend on its feed
  being open (after a cold start no feed is): the core works from the active
  episode.
- Pressing play on an episode that is still streaming with no download queued or
  underway (never started, or failed) queues it, as starting an episode does.
- A file the engine can't use is replaced by a fresh download once. If the fresh
  file is unusable too, the file itself is bad, so the download is marked failed
  (with the reason beside Retry), playback carries on from the stream, and
  pressing play doesn't fetch it again (otherwise stream, download, swap, fail
  would repeat for as long as playback went on).
- That memory is per playback session and clears when a local file loads; after a
  relaunch the cycle can run a couple of times before it stops. The bad file
  stays on disk until a retry overwrites it.

## Consequences

- The swap is audible: a brief gap, and up to about a second of repeated audio.
  See "Audible stream-to-local swap" in [[player-streaming]] for the options.
- The bad-file memory is set at the *first* failure, so a replacement that fails
  for an unrelated reason isn't retried on play for the rest of the session. See
  [[player-streaming]].
