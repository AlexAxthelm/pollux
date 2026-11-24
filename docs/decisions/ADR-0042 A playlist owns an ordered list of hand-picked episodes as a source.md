---
type: decision
status: accepted
feature:
  - "[[playlists]]"
  - "[[playlists-rules]]"
  - "[[playlists-playback]]"
  - "[[playlists-editor]]"
related:
  - "[[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]"
  - "[[US-0248 Be warned when an episode I add is hidden by the rules]]"
  - "[[ADR-0027 A playlist is a query over sources]]"
  - "[[US-0202 Add episodes to playlists]]"
  - "[[US-0204 Episodes can be reordered in playlists]]"
  - "[[US-0242 Order a playlist by hand with top and end pools]]"
  - "[[US-0243 See when a filter hides a hand-picked episode]]"
---
# A playlist owns an ordered list of hand-picked episodes as a source

## Context

Some episodes need to be in a playlist even though no show or playlist source would put them there. [[US-0202 Add episodes to playlists]] asks for this. A playlist stays a query ([[ADR-0027 A playlist is a query over sources]]), so these episodes have to be a source, not a stored order. Moving episodes to the top or end is a separate matter, decided in [[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]].

## Decision

A playlist can own a **hand-picked list**: single episodes that were added by hand. It is a source in the query, but it exists only inside its playlist. It is not selectable as a source by other playlists, and it does not appear in the library or the Feed Selector. The editor shows it as a row in the sources, such as "12 other episodes", that can be ordered like any other source and tapped to edit ([[playlists-editor]]).

- Its natural order is the order the episodes were added, oldest first, so each member stores when it was added ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]).
- **Filters apply to hand-picked episodes as normal.** A filter can remove one. The user needs to be able to see that it has happened ([[US-0243 See when a filter hides a hand-picked episode]]).
- Hand-picked lists are not MVP, as [[playlists-playback]] already says.

## Consequences

- "Add to playlist" in the episode menu, and **+ Episode** in the editor, add an episode to the hand-picked list ([[episode]], [[playlists-editor]]).
- `DATA_MODEL` gains a hand-picked source: a playlist-owned list of episode ids, each with the time it was added.
- Still pure query: the hand-picked list is one more `FROM`.
- **Storage:** hand-picked episodes count as members of the playlist. Where the playlist requests retention or downloads ([[ADR-0044 An episode's settings come from the context it is played from]], [[storage-eviction]], [[US-0044 Retention rules interactions]]), those requests cover its hand-picked episodes too, so a hand-picked episode of a show that keeps none is still retained while the playlist asks for it.
- **Unsubscribe:** when a show is unsubscribed its episodes are deleted, and any hand-picked members for those episodes are dropped in the same transaction, so the list never holds an id with no episode. See [[subscription-unsubscribe]].
- A hand-picked row can say where it was added from (for example "added from playlist 'foobar favorites'"), a small nicety like the queue's `from:` line ([[ADR-0046 Queued episodes play before the active playlist resumes]]). Not specified further.
- Open for the build: how the hand-picked list is edited when its row is tapped, and how to remove one episode from it.
