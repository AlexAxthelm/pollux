---
type: decision
status: accepted
feature:
  - "[[playlists]]"
  - "[[playlists-rules]]"
  - "[[playlists-circular-dependencies]]"
related:
  - "[[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]]"
  - "[[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]"
  - "[[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]"
  - "[[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]"
---
# Resolution is bounded by the work it does, and playlists have no episode limit

## Context

[[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]] gave every playlist an episode limit (default 1,000, a per-playlist override, a hard maximum of 50,000). It began as the guard against loops between playlists. Loops are now handled by leaving out a playlist already on the path ([[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]), so the limit became a size cap. As a size cap it overlaps the **First N episodes** filter, which is the same cut applied by choice; it does not reduce the work, because sorting and filtering still look at every source episode; and it silently truncates an included playlist, so an outer oldest-first playlist can lose old episodes that raising its own limit cannot bring back. What resolution really needs is protection from a runaway, because it also runs when the player is restored at launch, where a hang would be worse than a cut list.

## Decision

- **Playlists have no episode limit.** There is no setting, no per-playlist override and no "Showing first N episodes." message. A user who wants a cap adds **First N episodes** to the rules, at the end of the stack for a cap on the final list ([[playlists-rules]]).
- **A resolution is bounded by the work it does.** It counts the playlists it expands and the episodes it handles (the second matters when Deduplicate is off, because included playlists can then multiply a list without a loop). The count covers the whole root request, nested work included, and not each included playlist on its own. When the count passes a fixed bound, the resolution **fails**: the playlist shows "This playlist could not be loaded". There is no partial result. Nothing is shown from it as if it were the playlist, played from it, or used for pin cleanup, and an included playlist that fails makes the playlist that includes it fail too. A partial list could be in the wrong order, could contain episodes a filter excludes, and could make a pin look as if no source provides its episode.
- **The bound is set high**, so that it is rarely met in practice: it is a guard against a runaway, and meeting it means something is wrong. The number is chosen by performance testing, with a worst-case library in mind (for example 200 shows with 500 episodes each, through All shows). It is one number, used by the preview, playback and restore, so that a preview always predicts playback.
- **Subscriptions and the All shows source are not capped.** A playlist can therefore be very long. Lists are shown and played lazily, so length alone is not a problem.
- Caching playlists that are outside any loop is an optimisation for the build, not a rule here. How the cost of loops grows, and the smarter budget to decide when import is built, are in [[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]].

## Consequences

- This supersedes [[ADR-0030 Playlist resolution is capped by a per-playlist episode limit]]: the limit, its setting and its message go. The "Episode limit" setting in [[settings]] and the editor's Settings section are removed.
- Nothing cuts an included playlist unless its own rules say so, so the silent truncation that ADR-0030 accepted is gone. End-pinned episodes ([[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]) are no longer cut by a limit.
- With Deduplicate off, repeated copies count toward the work bound.
- The "three kinds of limit" overlap is gone: First N episodes is the only cut by count.
- Open for the build: the bound's number, set by performance testing.
