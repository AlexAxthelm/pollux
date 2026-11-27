---
type: decision
status: accepted
feature:
  - "[[playlists]]"
  - "[[playlists-rules]]"
  - "[[playlists-circular-dependencies]]"
related:
  - "[[ADR-0030 Playlist resolution is capped at 1,000 episodes instead of detecting cycles]]"
  - "[[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]"
  - "[[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]"
  - "[[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]"
---
# Resolution is bounded by the work it does, and playlists have no episode limit

## Context

[[ADR-0030 Playlist resolution is capped at 1,000 episodes instead of detecting cycles]] capped every playlist at 1,000 items instead of detecting cycles, and said the cap should be configurable in advanced settings. Loops are now handled by leaving out a playlist already on the path ([[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]]), so the cap no longer guards against cycles and is only a size cap. As a size cap it overlaps the **First N episodes** filter, which is the same cut applied by choice; it does not reduce the work, because sorting and filtering still look at every source episode; and a cap on an included playlist would silently truncate it, so an outer oldest-first playlist could lose old episodes that raising its own cap cannot bring back. What resolution really needs is protection from a runaway, because it also runs when the player is restored at launch, where a hang would be worse than a cut list.

## Decision

- **Playlists have no episode limit.** There is no setting and no "Showing first 1,000 episodes." message. A user who wants a cap adds **First N episodes** to the rules, at the end of the stack for a cap on the final list ([[playlists-rules]]).
- **A resolution is bounded by the work it does.** It counts the playlists it expands and the episodes it handles (the second matters when Deduplicate is off, because included playlists can then multiply a list without a loop). The count covers the whole root request, nested work included, and not each included playlist on its own. When the count passes a fixed bound, the resolution **fails**: the playlist shows "This playlist could not be loaded". There is no partial result. Nothing is shown from it as if it were the playlist, played from it, or used for pin cleanup, and an included playlist that fails makes the playlist that includes it fail too. A partial list could be in the wrong order, could contain episodes a filter excludes, and could make a pin look as if no source provides its episode.
- **The bound is set high**, so that it is rarely met in practice: it is a guard against a runaway, and meeting it means something is wrong. The number is chosen by performance testing, with a worst-case library in mind (for example 200 shows with 500 episodes each, through All shows). It is one number, used by the preview, playback and restore, so that, for the same query and the same anchor context, a preview agrees with playback. Playback can resolve with an anchor, which can change intermediate lists and so their work count, so identity in general is not promised.
- **Subscriptions and the All shows source are not capped.** A playlist can therefore be very long. Lists are shown and played lazily, so length alone is not a problem.
- Caching playlists that are outside any loop is an optimisation for the build, not a rule here, with two limits. A cached **final list** cannot be reused when an anchor is present, because the anchored episode must pass a status filter that it would fail in an ordinary resolution (a not-played playlist that has just lost the finished episode), so anchor-dependent resolution bypasses it; caching source data or context-independent stages is fine. And the work count is taken as if there were no cache, so that a warm and a cold resolution of the same query agree, and a preview still predicts playback. How the cost of loops grows, and the smarter budget to decide when import is built, are in [[ADR-0048 Resolution leaves out a playlist already on the path, and the editor asks before a loop is made]].

## Consequences

- This supersedes the cap in [[ADR-0030 Playlist resolution is capped at 1,000 episodes instead of detecting cycles]] (its handling of loops is superseded by ADR-0048): the cap, its "Limit for playlist episodes" setting and its "Showing first 1,000 episodes." message go. The "Episode limit" setting in [[settings]] and the editor's Settings section are removed.
- Nothing cuts an included playlist unless its own rules say so, so the silent truncation a cap on an included playlist would cause never arises. End-pinned episodes ([[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]) are no longer cut by a limit.
- With Deduplicate off, repeated copies count toward the work bound.
- The "three kinds of limit" overlap is gone: First N episodes is the only cut by count.
- Open for the build: the bound's number, set by performance testing.
