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

#### Identity across refreshes

An Episode is matched to its existing row by `(subscription_id, feed_guid)`,
where `feed_guid` comes from the feed's `<guid>`. This key decides whether a
refresh **updates** an episode or **inserts a new one**, so it also decides
whether playback position, played status, and flags survive a refresh. Two open
problems sit here.

**Unstable identity when `<guid>` is absent.** `<guid>` is optional in RSS 2.0.
When it is missing, `feed-rs` synthesises a random UUID that differs on every
parse — parsing one unchanged feed twice yields two different ids. Because the
unique key then never matches, every refresh re-inserts every episode as new
rows: the library grows without bound and playback state resets to Unplayed,
since the rows carrying it are orphaned rather than updated. This is a live
defect, not a hypothetical.

The fix needs an identity that is *stable* across refreshes and *unique* within
a feed. Candidates and their failure modes:

- **Enclosure URL** — stable for many feeds, but dynamic ad insertion and
  tracking prefixes rewrite it, which reproduces the same duplication
- **Normalised enclosure URL** (query string and known tracking prefixes
  stripped) — better, still breaks if the path itself changes
- **Title + publication date** — survives URL changes, breaks on an edited
  title, and collides for entries with no date
- **Detect the synthesised id** (e.g. `entry.id` parsing as a bare UUID) and
  fall back to one of the above — cheap, but misfires on feeds that legitimately
  use UUIDs as guids
- **Read `<guid>` ourselves** rather than relying on `feed-rs`'s `id`, so guid
  presence is known rather than inferred — most correct, most work

Choosing between these wants evidence rather than argument: how many real feeds
omit `<guid>`, and how many rewrite enclosure URLs between fetches. Neither is
known yet.

**Duplicate guids.** Feeds sometimes repeat a `<guid>` across entries. Storage
resolves this by overwriting, and because feeds are conventionally newest-first,
an unguarded overwrite let an *older* duplicate replace a *newer* episode.
`parse_feed` now collapses duplicates before storage, keeping the **first**
occurrence. That choice is deliberate — record it here so it is not silently
reversed.

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

Settings follow the cascading defaults pattern (global → subscription →
playlist → episode) documented in `DESIGN_PRINCIPLES.md` §3 and `GLOSSARY.md`.

**Resolved: settings are structured config objects, not key-value rows.**

Each level of the hierarchy holds a settings object of a shared shape. At the
app-default level all fields are concrete (with baked-in defaults). At every other
level, fields are optional — `None` means "inherit from the level above." The
cascade resolves a concrete value for every field by walking up the chain until
a non-None value is found, falling back to the global default.

Conceptually:

```
AppDefaultSettings      — all fields concrete; the authoritative defaults
GlobalSettings      — Same fields, all optional; overrides app defaults
SubscriptionSettings — same fields, all optional; overrides global
PlaylistSettings    — same fields, all optional; overrides global
                      (not subscription — a playlist may span many subscriptions)
EpisodeSettings     — same fields, all optional; overrides subscription or playlist
                      depending on playback context
```

These names are placeholders. The final names and the Rust representation
(trait, generic struct, or otherwise) will be settled during implementation.
The key constraint is that all levels share the same field shape so that the
cascade logic is uniform.

See also: `docs/features/settings.md`, `DESIGN_PRINCIPLES.md` §3.
