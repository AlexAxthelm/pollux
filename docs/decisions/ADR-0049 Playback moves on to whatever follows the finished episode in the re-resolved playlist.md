---
type: decision
status: accepted
feature:
  - "[[playlists-playback]]"
related:
  - "[[ADR-0029 The active playlist is set by where playback started]]"
  - "[[ADR-0006 No auto-advance until playlists exist]]"
  - "[[ADR-0046 Queued episodes play before the active playlist resumes]]"
  - "[[US-0139 Auto play next episode]]"
---
# Playback moves on to whatever follows the finished episode in the re-resolved playlist

## Context

A playlist is a query, so its list changes while the user listens. A playback-status filter such as "unplayed" removes the current episode as soon as it is played. Episodes can arrive, and the user can edit the playlist while it plays. "Next" has to mean something in each of these cases. [[ADR-0006 No auto-advance until playlists exist]] was written as a temporary rule until playlists could define a next episode.

## Decision

Each time playback moves on automatically, the playlist is **resolved again** and the next episode is the one that comes **after the episode that just finished, in the active playlist's order**. The finished episode is treated as still being in its place, so being marked played does not make it drop out first.

- Starting an episode from the middle of a playlist makes that playlist active and plays from that episode's position ([[ADR-0029 The active playlist is set by where playback started]]).
- Episodes added right after the current one play next. If the current episode is moved to the top, whatever follows it in the new order plays next.
- Queued episodes play before this, as in [[ADR-0046 Queued episodes play before the active playlist resumes]].
- The playlist becomes inactive when nothing follows the finished episode.
- The alternate setting "play next: from top" plays the first episode in the re-resolved list instead of the one after the finished episode ([[playlists-playback]]).

## Consequences

- Resolution runs on each advance, so it has to be fast for lists up to the episode limit ([[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]]).
- The auto-advance in [[ADR-0006 No auto-advance until playlists exist]] is replaced by this rule once playlist playback is built. That ADR is left as it stands until then.
- #needs-decision **What follows an episode that an edit removed from the list?** If the user edits the playlist so that the current episode is no longer in it, there is no position to continue from. Not decided.
