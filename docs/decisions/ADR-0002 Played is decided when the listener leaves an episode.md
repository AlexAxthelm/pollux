---
type: decision
status: accepted
feature:
  - "[[player-progress]]"
related:
  - "[[US-0155 Set played threshold]]"
  - "[[ADR-0005 Position persistence cadence and resume rewind]]"
---
# Played is decided when the listener leaves an episode

## Context

Progress is saved continuously, and the last seconds of an episode are often
outro the listener may or may not want. Deciding "played" as a side effect of
saving progress would cut short someone listening on through the end with the
screen locked.

## Decision

**Played tolerance.** Played is decided when the listener *leaves* an episode,
never as a side effect of saving progress. It happens when the engine reaches the
end, when they pause, or when they start another episode, while within **15s** of
the end (`PLAYED_TOLERANCE_SECS`, but never more than a quarter of a short
episode). The status becomes played, the stored position resets to 0, the saved
context is cleared and the player goes inactive. The engine's duration wins over
the feed's ([[ADR-0010 The engine's duration beats the feed's]]); with no known
duration only the engine reaching the end finishes an episode.

**Not leaving.** Periodic saves and backgrounding only record in-progress and a
position, so listening on through the last seconds with the screen locked is
never cut short. A system pause (a call, Siri, unplugged headphones) keeps the
episode and its place and never counts as finishing, so it can resume and play
out the end. When an audio-session interruption (a call, Siri) ends, playback
resumes only if that interruption is what paused it and the system says it may:
the system also says "may resume" after interruptions that found playback
already paused, and the listener's pause then stands. A pause from unplugged
headphones is never resumed automatically, and playing or scrubbing by hand
during an interruption cancels the auto-resume.

## Consequences

- The 15s tolerance is hard-coded. It *will become a user setting* (see
  [[US-0155 Set played threshold]]).
- The interruption rules are the playback side of [[player-os-integration]].
