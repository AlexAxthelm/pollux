---
type: decision
status: accepted
feature:
  - "[[player]]"
related:
  - "[[ADR-0003 Streaming starts at once and swaps to the download]]"
---
# Engine events carry a session id

## Context

The core owns all player policy (`shared/src/player.rs`); the shell's
`PlaybackManager` only drives the audio engine. The engine reports ticks, ends
and failures asynchronously, so a report can arrive after the item it describes
has already been replaced.

## Decision

Each load into the engine gets a session id, and everything the engine reports
carries it. The core ignores news from any other session, so a late tick, end or
failure from a replaced item (a source swap, a retry, another episode) can't
affect the current one.

## Consequences

- A hand-over that wants to continue seamlessly can adopt a new session when it
  cues; the old stream item's late end-of-file carries the old session and is
  ignored (see the options in [[player-streaming]]).
