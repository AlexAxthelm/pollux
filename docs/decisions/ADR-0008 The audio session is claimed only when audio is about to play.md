---
type: decision
status: accepted
feature:
  - "[[player-os-integration]]"
related: []
---
# The audio session is claimed only when audio is about to play

## Context

Claiming the audio session interrupts other apps' audio. A paused load (a source
swap, scrubbing a restored episode) shouldn't cut off music the listener has
moved to, and AVPlayer keeps its rate when its item is replaced.

## Decision

The audio session is claimed only when audio is about to play: starting or
resuming does, but a paused load does not. A paused load also pauses the player,
because AVPlayer keeps its rate when its item is replaced.

The session is released when playback stops, so other apps are told they may
resume. The system often refuses that while the removed item's I/O is still
winding down, so a refused release is tried again shortly (five tries in all,
logged if it never works), and claiming the session again cancels a pending try.

## Consequences

- The retry's effect on real iOS is only checked on a device: play an episode to
  the end with music paused in another app, and see that the music resumes.
