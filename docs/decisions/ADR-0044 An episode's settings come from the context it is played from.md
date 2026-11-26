---
type: decision
status: accepted
feature:
  - "[[settings-cascade]]"
related:
  - "[[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]"
  - "[[ADR-0028 A subscription is an implicit single-feed playlist]]"
  - "[[ADR-0029 The active playlist is set by where playback started]]"
  - "[[ADR-0053 Resolution is bounded by the work it does, and playlists have no episode limit]]"
  - "[[US-0167 Playlist settings]]"
  - "[[US-0170 Playlists inherit from show default settings]]"
---
# An episode's settings come from the context it is played from

## Context

Settings can be set globally, per subscription, per playlist (playlists can include other playlists) and per episode. An episode can belong to a subscription and to many playlists, so "which settings apply" can't be read off the episode alone.

## Decision

Settings resolve along the **context stack**, and the most local set value wins:

`global → subscription → included playlists (deepest first) → active playlist → episode`

The **active playlist** (the one playback was started from) is the most local context. Working up from the episode, the settings are looked up in the active playlist, then each playlist it includes on the route to the episode, then the episode's subscription, then global.

- The **active playlist** is still set by where playback started ([[ADR-0029 The active playlist is set by where playback started]]), and "next up" is whatever is next in it. Each episode it plays inherits its settings through the chain from that context.
- The **context** is the path playback took to reach the episode: the playlist the user started from, then each included playlist down to the one the episode came from. The playlist the user started from is the most local, and the playlists it includes sit above it, so starting from B gives B's values where starting from A gives A's. Starting from a subscription (an implicit playlist, [[ADR-0028 A subscription is an implicit single-feed playlist]]) gives `global → subscription → episode`. Starting from the library (the All subscriptions convenience playlist) gives the same, since convenience playlists have no settings of their own ([[playlists-convenience]]).
- The episode's own subscription is always in the stack, above all the playlists. A playlist value that is set beats the show; an unset playlist value falls through to the next playlist up the route, then the show, then global.
- An episode override beats everything, including explicit playlist values.
- Example: A includes B, A sets skip to 60s and B sets it to 20s. Played from A, the episode skips 60s; played from B, it skips 20s. Where A leaves a value unset, B's (or C's, for episodes via C) applies, so sibling sources in one playlist can give their episodes different effective settings.
- Each occurrence of an episode in a playlist's resolved list uses **its own route**. A playlist's Deduplicate filter is on by default ([[playlists-rules]]), so normally only one occurrence survives: the first at the point where Deduplicate sits in the playlist's rules, which may be a hand-picked copy or one from an included playlist. With Deduplicate off, each copy plays with the settings of its own route. Cycles end because a playlist already on the resolution path is left out ([[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]), and a bound on the work a resolution does stops a runaway ([[ADR-0053 Resolution is bounded by the work it does, and playlists have no episode limit]]).
- **The route and the resolved values are taken when an episode starts**, and are stored with the play context (so a cold start restores them). The player uses that copy and does not re-resolve on its own. Two things can change it while the episode plays:
  - A settings edit at a level **on the playing episode's route** (for example a skip time set on show S while E plays via `[A, S]`) re-resolves along the same route and applies at once.
  - Anything that would change the route itself (reordering sources, removing a source or the playlist, Deduplicate keeping a different occurrence) does not apply until the next episode starts. The playing episode keeps its route, and a level in it that no longer exists keeps its last value.
- **Next-episode behavior is not resolved along the route.** "Play next" or "From the top" belongs to the context playback started from, because "next" is defined by that playlist ([[ADR-0029 The active playlist is set by where playback started]], [[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]]). It resolves as the active playlist's value (or the active subscription's), else global. It skips included playlists and, for a playlist, the show level.
- A playlist's settings page cannot show one inherited value, since it varies by source. It shows "Follows each source" for an unset value, and a reset affordance for a set one. A later improvement: when every source agrees, show the one value with an inherited indicator.

Which settings exist at which level is declared per setting. Playback-time settings (skip, speed, rewind, next-episode behavior) apply to playlists and subscriptions. Storage, download and refresh settings are also valid on playlists, not only subscriptions, and are combined by storage rather than by the context stack (see Consequences) (for example: a show keeps 1 episode, but a playlist makes everything available before a trip). Very few settings are not applicable to a playlist.

## Consequences

- Playback must carry the path, not just the active playlist id ([[ADR-0029 The active playlist is set by where playback started]]): resolution has to return, for each episode, the route it was reached by. This is more work than entry-point-only and is the main cost of this decision.
- The resolver is one function over the stack, and an unset value is `None` at every level, as [[DATA_MODEL]] already says; [[DATA_MODEL]] has been updated to match (the subscription sits under the playlist).
- Storage, download and refresh settings are **requests, not context lookups**. A playlist is evaluated like a subscription: each one that contains an episode can request that it be downloaded or retained, and the storage rules combine those requests (default: retain), as in [[storage-eviction]] and [[US-0044 Retention rules interactions]]. They don't resolve along the context stack, because there's no playback context for them. The setting values themselves (the "keep 1" or "keep all" fields) still follow the cascade at each owner.
- Changing speed or skip from the player affects the current player session only for now; saving it to the episode or context is deferred.
- Revisit if the full-path cost proves too high; entry-point-only is the cheaper fallback.
