---
type: decision
status: accepted
feature:
  - "[[player]]"
related: []
---
# Playback errors pause playback and retry on play

## Context

Playback can fail for reasons outside the app's control (offline, another app
holding the audio session, a bad stream).

## Decision

A playback failure leaves playback paused with a transient banner in the player
and mini-player. Pressing play retries.

## Consequences

- Verified on a device: the offline error banner stays until the next successful
  action, which is accepted.
- The failed-download triangle and Retry stay on the episode row while the banner
  is showing.
