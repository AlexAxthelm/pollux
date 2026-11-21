---
type: decision
status: draft
feature:
  - "[[settings-cascade]]"
related:
  - "[[ADR-0028 A subscription is an implicit single-feed playlist]]"
  - "[[ADR-0029 The active playlist is set by where playback started]]"
  - "[[ADR-0030 Playlist resolution is capped at 1,000 episodes instead of detecting cycles]]"
  - "[[US-0167 Playlist settings]]"
  - "[[US-0170 Playlists inherit from show default settings]]"
---
# An episode's settings come from the context it is played from

## Context

Settings can be set globally, per subscription, per playlist (playlists can include other playlists) and per episode. An episode can belong to a subscription and to many playlists, so "which settings apply" can't be read off the episode alone. Notes disagreed about the chain (see `#needs-decision` in [[GLOSSARY]], [[DATA_MODEL]] and [[DESIGN_PRINCIPLES]]). Agreed in conversation with the maintainer; not yet reviewed as an ADR.

## Decision

Settings resolve along the **context stack**, and the most local set value wins:

`global → subscription → playlist (outermost) → … → playlist (innermost) → episode`

- The **context** is the path playback took to reach the episode: the playlist the user started from, then each included playlist down to the one the episode came from. Starting from a subscription (an implicit playlist, [[ADR-0028 A subscription is an implicit single-feed playlist]]) gives `global → subscription → episode`. Starting from the library gives the same, since the library has no settings of its own.
- The episode's own subscription is always in the stack, directly above the playlists. A playlist value that is set beats the show; an unset playlist value falls through to the show, then global.
- An episode override beats everything, including explicit playlist values.
- Sibling sources in one playlist (B and C inside A) can therefore give their episodes different effective settings.
- An episode reached by more than one route uses the route of its first occurrence in resolution order, as in [[ADR-0030 Playlist resolution is capped at 1,000 episodes instead of detecting cycles]]. This also bounds cycles.
- A playlist's settings page cannot show one inherited value, since it varies by source. It shows "Follows each source" for an unset value, and a reset affordance for a set one.

Which settings exist at which level is declared per setting. Playback-time settings (skip, speed, rewind, next-episode behavior) apply to playlists and subscriptions. Storage, download and refresh settings are also valid on playlists, not only subscriptions, and are combined by storage rather than by the context stack (see Consequences) (for example: a show keeps 1 episode, but a playlist makes everything available before a trip). Very few settings are not applicable to a playlist.

## Consequences

- Playback must carry the path, not just the active playlist id ([[ADR-0029 The active playlist is set by where playback started]]): resolution has to return, for each episode, the route it was reached by. This is more work than entry-point-only and is the main cost of this decision.
- The resolver is one function over the stack, and an unset value is `None` at every level, as [[DATA_MODEL]] already says; [[DATA_MODEL]] needs updating (subscription now sits under playlist).
- Storage, download and refresh settings are **requests, not context lookups**. A playlist is evaluated like a subscription: each one that contains an episode can request that it be downloaded or retained, and the storage rules combine those requests (default: retain), as in [[storage-eviction]] and [[US-0044 Retention rules interactions]]. They don't resolve along the context stack, because there's no playback context for them. The setting values themselves (the "keep 1" or "keep all" fields) still follow the cascade at each owner.
- Changing speed or skip from the player affects the current player session only for now; saving it to the episode or context is deferred.
- Revisit if the full-path cost proves too high; entry-point-only is the cheaper fallback.
