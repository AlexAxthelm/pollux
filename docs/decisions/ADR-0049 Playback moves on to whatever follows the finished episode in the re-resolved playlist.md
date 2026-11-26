---
type: decision
status: accepted
feature:
  - "[[playlists-playback]]"
related:
  - "[[US-0250 Be warned before removing the playing episode from its playlist]]"
  - "[[ADR-0029 The active playlist is set by where playback started]]"
  - "[[ADR-0006 No auto-advance until playlists exist]]"
  - "[[ADR-0044 An episode's settings come from the context it is played from]]"
  - "[[ADR-0046 Queued episodes play before the active playlist resumes]]"
  - "[[US-0139 Auto play next episode]]"
---
# Playback moves on to whatever follows the finished episode in the re-resolved playlist

## Context

A playlist is a query, so its list changes while the user listens. A playback-status filter such as "unplayed" removes the current episode as soon as it is played. Episodes can arrive, and the user can edit the playlist while it plays. "Next" has to mean something in each of these cases. [[ADR-0006 No auto-advance until playlists exist]] was written as a temporary rule until playlists could define a next episode.

## Decision

Each time playback moves on automatically, the playlist is **resolved again** and the next episode is the one that comes **after the episode that just finished, in the active playlist's order**. The list is resolved at **the last moment the finished episode was still active**: just before it is marked played, and with every edit made while it played already in effect. "Next" is read from that list, so being marked played does not make the episode drop out first. It is not read from the moment the episode was loaded.

**Worked examples.** The playlist has an "unplayed" filter, so each episode drops out of the list once it is played.

- The list is `[A, B, C, D, E]` and C is playing. When C finishes, D plays next. If, while C played, the user rearranged it to `[A, C, E, D, B]`, then E plays next: C is still in the list at that moment, and E is what follows it.
- With Deduplicate off an episode can appear twice, and the playing copy is the one with the same route. The list is `[A, B, C1, D, E, C2, F, G]` and C1 is playing: D plays next. If it is rearranged to `[C2, A, B, D, E, F, C1, G]`, then G plays next, because C1 is the copy that is playing and G follows it. C2 is not.

- Starting an episode from the middle of a playlist makes that playlist active and plays from that episode's position ([[ADR-0029 The active playlist is set by where playback started]]).
- Episodes added right after the current one play next. If the current episode is moved to the top, whatever follows it in the new order plays next.
- Queued episodes play before this, as in [[ADR-0046 Queued episodes play before the active playlist resumes]].
- The playlist becomes inactive when nothing follows the finished episode.
- **If the playing episode is no longer in the list for any reason** (an edit, a feed refresh, being pushed past First N episodes, being marked played by hand), nothing changes while it plays: it keeps playing with its current settings. When it finishes the list is resolved at that moment, and if the episode is not in it, playback ends. If it left and came back before it finished, it is in the list and playback continues after it. The playlist is not a context for anything after it.
- **The next episode must be in the resolved list.** The resolver runs at that moment, so an episode that no longer qualifies is never chosen, and the chosen episode is not checked again afterwards. If the same episode appears twice and the first copy finishes, the second copy plays.
- **If the active playlist is deleted**, it is the same case as an edit that removes the playing episode: the playing episode finishes with the context it started with, and playback then ends. The queue keeps playing, because it is not part of the playlist ([[ADR-0046 Queued episodes play before the active playlist resumes]]), and playback ends when it empties, since there is no playlist to resume. Other playlists that include the deleted one drop it as a source. The delete confirmation says when the playlist is playing ("It is playing now; it will finish this episode and stop").
- **If the app closes and restarts before the playing episode has finished**, the episode is reopened with its stored context (the route and resolved settings), also when its playlist has since been deleted or changed. Restoring an active context restores that context. Once the episode finishes, playback ends as above.
- Pausing near the end of an episode does not finish it; see [[ADR-0052 Pausing near the end does not mark an episode played]] (draft).
- **Edits take effect on Save.** A change to sources or rules first changes playback at the next automatic advance. A change to settings shows in the editor straight away; it reaches the playing episode at once only if it is at a level on that episode's route, and otherwise from the next episode ([[ADR-0044 An episode's settings come from the context it is played from]]).
- **Which copy of the finished episode counts.** With Deduplicate off an episode can appear twice. "After the finished episode" means after the occurrence with the same episode and route, or after the first occurrence if that route is gone.
- The alternate setting "From the top" plays the first **unplayed** episode (not played, [[playlists-rules]]) in the list resolved at that moment, counting the episode that just finished as played so it is not picked again, instead of the one after the finished episode, and playback ends if every episode is played. Skipping played episodes is what stops it replaying the first episode forever ([[playlists-playback]], [[US-0254 Be warned about settings that make a playlist replay]]).

## Consequences

- Resolution runs on each advance, so it has to be fast for lists up to the episode limit ([[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]]).
- The auto-advance in [[ADR-0006 No auto-advance until playlists exist]] is replaced by this rule once playlist playback is built. That ADR is left as it stands until then.
- The playing episode's context route is fixed when it starts and stays fixed until it ends ([[ADR-0044 An episode's settings come from the context it is played from]]), so an edit that changes the route cannot change its settings mid-episode. A settings edit at a level on the route does reach it at once, which re-resolves along the same route.
