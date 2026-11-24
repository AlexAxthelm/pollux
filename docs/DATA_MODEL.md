---
type: reference
---
# Pollux Data Model

This document defines the core entities and their relationships. It is the
code-facing complement to [[GLOSSARY]], which covers user-facing terminology.

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

### Subscription *(Sketched)*

The app's record of a podcast the user has subscribed to, backed by a feed URI.
One-to-many with Episode.

See also: [[GLOSSARY]] — Subscription.

#### Refresh state

Besides its identity and feed metadata, a Subscription carries the bookkeeping
feed refresh needs (see [[subscription-refresh]], "As built"):

- `last_refreshed` — when the feed was last successfully fetched (a 304 counts).
- `etag`, `last_modified` — validators from the last successful fetch, replayed
  as a conditional GET.
- `last_refresh_error` — why the most recent refresh failed; cleared on success.
- `retry_after_until` — unix time before which *automatic* refresh leaves the
  feed alone. Set from `Retry-After` on a 429, or to a short backoff after any
  other failure. Manual refresh ignores it.

Refresh outcomes that produce no new body (304, 429, failures) are written by
`UpdateRefreshState`; a 200 goes through the normal feed upsert, which overwrites
all of the above from the fresh parse. A 304's write also flags episodes whose
`missing_since` clock has run the grace period, and reports how many.

### Episode *(Sketched)*

The atomic unit of content. Belongs to a Subscription. Has playback status,
download status, and flag state.

See also: [[GLOSSARY]] — Episode, Playback status, Flag.

#### What a refresh preserves

On a conflict, the episode upsert refreshes feed-owned metadata (title,
description, enclosure URL, artwork, pub date, duration) and **keeps** the row's
`id`, playback state, flags, `download_status` and `local_path`. Two fields
need care:

- `file_size_bytes` — for a `Downloaded` episode this is the real on-disk size
  recorded by the download path, so a refresh keeps it rather than overwriting it
  with the feed's advertised `<enclosure length>`. For every other episode the
  feed's value is the freshest available and is written.
- `download_status` — an episode flagged `RemovedFromFeed` that reappears in the
  feed returns to `NotDownloaded`. Episodes that drop out of the feed are flagged
  only after being absent for 48 hours, and only if they are `NotDownloaded` or
  `Failed`; see [[subscription-delisted-episodes]].
- `missing_since` — storage-only (the core never sees it): the unix time an
  episode was first missed by a refresh, or NULL while it is present. Every
  refresh with a non-empty feed starts a clock for each of the subscription's
  episodes that doesn't have one, and the upsert clears it for the ones still
  present, so only absent episodes keep it. An episode is flagged once its clock
  is at least `removalGraceSeconds` (48 hours) old.

#### Schema baseline

See [[ADR-0040 The database schema is a single baseline until the first release]].

#### Identity across refreshes

An Episode is matched to its existing row by `(subscription_id, feed_guid)`,
where `feed_guid` is the feed's `<guid>` when present. This key decides whether a
refresh **updates** an episode or **inserts a new one**, so it also decides
whether playback position, played status, and flags survive a refresh.

**Identity when `<guid>` is absent.** Resolved for RSS: see [[ADR-0038 A missing guid is replaced by an id derived from the enclosure URL and title]]. It also records the residual failure modes.

**Duplicate guids.** Collapsed to the first occurrence before storage: see [[ADR-0039 Duplicate guids collapse to the first occurrence before storage]].

### Playlist *(Identified)*

A query definition: one or more sources, with ordered filters and sorts applied.
Produces an ordered Episode list at query time — it is not a stored list of
episodes (apart from the hand-picked list below, which stores episode ids).

One-to-many with PlaylistSource, PlaylistFilter, PlaylistSort.

See also: [[GLOSSARY]] — Playlist.

### PlaylistSource *(Identified)*

A join entity between a Playlist and one of its sources. A source may be a
Subscription, another Playlist (enabling meta-playlists), or the convenience **All shows** source that follows the library. Carries an ordering
so that "by source" sorts are deterministic.

A further kind of source is the playlist's own **hand-picked list**: single episode ids,
each with the time it was added (the list's natural order, oldest first). It exists only
inside the playlist that owns it and cannot be used as a source by another playlist. Filters
apply to its episodes as normal. See [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]. *Not MVP.*

### PlaylistPin *(Identified)*

An occurrence of an episode that the user has moved to the top or the end of a playlist:
the playlist, the episode id, the source it came through, a group (top or end) and its place in
that group. It is independent of the source kinds, so a pin works on an episode from a show, an
included playlist or the hand-picked list, and only that occurrence moves. The Manual ordering sort uses the pins. See [[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]. *Not MVP.*

### PlaylistFilter *(Identified)*

A filter rule attached to a Playlist. Carries a type (e.g. playback status,
recency limit, Deduplicate), parameters, include/exclude polarity (not for
Deduplicate, which is on or off), and a position in the playlist's rule stack.

### PlaylistSort *(Identified)*

A sort rule attached to a Playlist. Carries a type (e.g. by age, by source, manual
ordering), parameters, direction (asc/desc), and a position in the same rule stack as
the filters. Rules are applied top to bottom, each to the result of the ones above it, so
the last sort applied is the main order ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]).

### PlayContext *(Identified)*

Represents the currently active playback state: which Playlist or Subscription
is active, which Episode is current, and the position within that episode.
Persists across app restarts so the mini-player can be restored on cold start.

User-facing term not yet settled — "Now Playing" is used informally in the
specs but has not been adopted as a formal term. Do not use "PlayContext" in
user-facing copy until this is resolved.

Relationships: references a Playlist or Subscription (via EpisodeSource), and
an Episode.

The queue ([[ADR-0046 Queued episodes play before the active playlist resumes]]) also persists
with the play context: an ordered list of entries, each an episode id plus the context route it was
queued from. *Not built yet.*

Stored in the single-row `play_context` table as the episode id plus the source as
a kind and an id (`Subscription` today). Position is not stored here: it lives on
the episode row, so the two can never disagree.

---

## Deferred Entities

The following entities are identified but intentionally not defined yet.

### PlayHistory

A continuous event log of listening activity: episode, position, device,
playback speed, wall time. Referenced in [[listening-stats]].
Required for listening statistics and will interact with sync. Detail deferred.

### Bookmark

A user-set timestamp on an episode, with an optional text note. Referenced in
[[bookmarks]]. Detail deferred.

### Device

Represents a physical device the app runs on. Required for per-device storage
rules and sync. Detail deferred until sync is scoped.

See also: [[storage-sync]], [[sync-downloads]], [[sync]].

---

## Internal Abstractions

### EpisodeSource *(placeholder name)*

The internal abstraction representing either a Subscription or a Playlist. Used
by the playback engine, rules engine, and playlist editor, which need to operate
on both without caring which kind they have.

This is a Rust trait or enum, not a database entity. The name is a placeholder —
the final name will be settled during implementation. Do not use in
user-facing copy.

Implemented as the `EpisodeSource` enum (`shared/src/domain/episode_source.rs`),
with one variant today, `Subscription { id }`, since a subscription is an implicit
single-feed playlist. A `Playlist { id }` variant joins when playlists exist. The
player already carries an `EpisodeSource` (not a bare subscription id) through its
state, the saved play context, and the "From:" row, so adding the variant needs no
schema or event change; only the places that resolve a source (its title, its
episode order) gain a case. The name is kept for now.

See also: [[GLOSSARY]] — EpisodeSource.

---

## Settings

Settings follow the cascading defaults pattern (global → subscription →
playlist(s) → episode) documented in [[DESIGN_PRINCIPLES]] §3 and [[GLOSSARY]]. The chain
an episode uses is the context it is played from: see
[[ADR-0044 An episode's settings come from the context it is played from]].

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
PlaylistSettings    — same fields, all optional; overrides subscription (a playlist
                      may span many subscriptions, so an unset value follows each
                      episode's own subscription). Playlists can nest: the playlist
                      playback started from is the most local, and the playlists it
                      includes sit above it, closer to the subscription
EpisodeSettings     — same fields, all optional; overrides everything above
```

These names are placeholders. The final names and the Rust representation
(trait, generic struct, or otherwise) will be settled during implementation.
The key constraint is that all levels share the same field shape so that the
cascade logic is uniform.

See also: [[settings]], [[DESIGN_PRINCIPLES]] §3.
