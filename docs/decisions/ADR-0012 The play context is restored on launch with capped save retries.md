---
type: decision
status: accepted
feature:
  - "[[player-progress]]"
related:
  - "[[ADR-0007 Playback source is separate from the episode's feed]]"
---
# The play context is restored on launch, with capped save retries

## Context

If the app closes while something is playing, the listener expects the
mini-player back at their place. The context is written once when playback
starts, so a failed write must not silently leave the wrong episode (or none) to
restore.

## Decision

**Restore.** An episode already marked played is not restored at launch (its
saved context is cleared); an in-progress one comes back paused at its place.

**Saving.** The context is written once when playback starts, so a failed write
is retried at every checkpoint until storage confirms it; otherwise a relaunch
would restore the wrong episode, or none. It is tried at most 5 times in all
(`CONTEXT_SAVE_ATTEMPTS`): a write that can't succeed isn't repeated for as long
as playback goes on, and the next episode starts afresh.

## Consequences

- Nothing is shown to the listener; the cost of giving up is that a relaunch may
  not restore this episode.
- A saved context is deleted with its episode (`ON DELETE CASCADE`); see the
  unsubscribe case in [[subscription-unsubscribe]].
