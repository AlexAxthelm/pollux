# Pollux Data Model

This document defines the core entities and their relationships. It is the
code-facing complement to `GLOSSARY.md`, which covers user-facing terminology.

This is a living document. Entities are added and refined as features are
specced and user stories are walked through. Field-level detail is added when
a feature is being actively implemented, not speculatively in advance.

Where an entity does not yet have a user-facing term settled, that is noted
explicitly.

---

## Status of this document

Entities listed here are at one of three levels of definition:

- **Identified** — we know this entity exists and roughly what it represents;
  fields and relationships not yet defined
- **Sketched** — key fields and relationships defined; detail may be incomplete
- **Defined** — field-level definition complete; ready to implement

---

## Core Entities

### Subscription *(Identified)*

The app's record of a podcast the user has subscribed to, backed by a feed URI.
One-to-many with Episode.

See also: `GLOSSARY.md` — Subscription.

### Episode *(Identified)*

The atomic unit of content. Belongs to a Subscription. Has playback status,
download status, and flag state.

See also: `GLOSSARY.md` — Episode, Playback status, Flag.

### Playlist *(Identified)*

A query definition: one or more sources, with ordered filters and sorts applied.
Produces an ordered Episode list at query time — it is not a stored list of
episodes.

One-to-many with PlaylistSource, PlaylistFilter, PlaylistSort.

See also: `GLOSSARY.md` — Playlist.

### PlaylistSource *(Identified)*

A join entity between a Playlist and one of its sources. A source may be a
Subscription or another Playlist (enabling meta-playlists). Carries an ordering
so that "by source" sorts are deterministic.

### PlaylistFilter *(Identified)*

A filter rule attached to a Playlist. Carries a type (e.g. playback status,
recency limit), parameters, include/exclude polarity, and an ordering (filters
are applied sequentially).

### PlaylistSort *(Identified)*

A sort rule attached to a Playlist. Carries a type (e.g. by age, by source),
parameters, direction (asc/desc), and an ordering (sorts are applied as
primary → secondary → etc.).

### PlayContext *(Identified)*

Represents the currently active playback state: which Playlist or Subscription
is active, which Episode is current, and the position within that episode.
Persists across app restarts so the mini-player can be restored on cold start.

User-facing term not yet settled — "Now Playing" is used informally in the
specs but has not been adopted as a formal term. Do not use "PlayContext" in
user-facing copy until this is resolved.

Relationships: references a Playlist or Subscription (via EpisodeSource), and
an Episode.

---

## Deferred Entities

The following entities are identified but intentionally not defined yet.

### PlayHistory

A continuous event log of listening activity: episode, position, device,
playback speed, wall time. Referenced in `docs/features/listening-stats.md`.
Required for listening statistics and will interact with sync. Detail deferred.

### Bookmark

A user-set timestamp on an episode, with an optional text note. Referenced in
`docs/features/bookmarks.md`. Detail deferred.

### Device

Represents a physical device the app runs on. Required for per-device storage
rules and sync. Detail deferred until sync is scoped.

See also: `docs/features/storage-sync.md`, `docs/features/sync.md`.

---

## Internal Abstractions

### EpisodeSource *(placeholder name)*

The internal abstraction representing either a Subscription or a Playlist. Used
by the playback engine, rules engine, and playlist editor, which need to operate
on both without caring which kind they have.

This is a Rust trait or enum, not a database entity. The name is a placeholder —
the final name will be settled during implementation. Do not use in
user-facing copy.

See also: `GLOSSARY.md` — EpisodeSource.

---

## Settings

Settings are discussed when features that require them are being specced.
The cascading defaults pattern (global → subscription → episode) is documented
in `DESIGN_PRINCIPLES.md` §3 and `GLOSSARY.md` — Cascading default. How
settings are represented in the data model (rows in a table vs. structured
config) is an open question.
