# Pollux Glossary

This document defines terminology used in Pollux's codebase, documentation,
and user-facing copy. It is intended as both a dictionary and a style guide —
where a term has user-facing implications, those are called out explicitly.

Terms will be added as the data model and feature set are defined. If a term
is not in this glossary, that is a signal it needs to be discussed and added,
not that it can be used freely.

For code-facing definitions and entity relationships, see `DATA_MODEL.md`.

---

## Content & Sources

- **Subscription**: the app's internal representation of a podcast the user
  has subscribed to, backed by an RSS/Atom/JSONFeed URI.
  - Related terms:
    - **Show**, **Podcast**: synonyms for subscription in user-facing copy.
      "Subscription" is generally preferred in UI text where precision matters
      (e.g. settings, unsubscribe flows); "show" or "podcast" are preferred
      in casual or onboarding contexts where "subscription" feels formal.
    - **Feed**: distinct from subscription. A feed is the underlying
      RSS/Atom/JSONFeed document that the app fetches. A subscription is the
      app's record of tracking that feed. Do not use "feed" as a synonym for
      subscription in documentation or code.
  - See also: `DATA_MODEL.md` — Subscription.

- **Playlist**: a user-created query over one or more sources, with optional
  filters and sorts applied. Produces an ordered episode list.
  - Related terms:
    - **Smart playlist**, **queue**: avoid these in user-facing copy. The word
      "playlist" covers all cases.
    - **Convenience playlist**: internal/doc term for system-provided playlists
      that require no configuration ("All Subscriptions", "Downloaded"). Not a
      user-facing distinction — these appear in the UI simply as playlists.
  - See also: `DATA_MODEL.md` — Playlist, PlaylistSource, PlaylistFilter,
    PlaylistSort.

- **EpisodeSource** *(placeholder — not a finalized term)*: the internal
  abstraction representing either a subscription or a playlist. Used in the
  playback engine, rules engine, and playlist editor, which need to operate on
  both without caring which kind they have. This term should not appear in
  user-facing copy. The right name for this concept will be settled when the
  data model is defined.
  - See also: `DATA_MODEL.md` — EpisodeSource.

---

## Navigation & Views

- **Library**: the home screen of the app, showing the user's subscriptions
  and playlists together in a single browsable view. User-facing term.
  - Related terms:
    - **Catalog**: do not use. Considered and rejected — it reads as a
      collective noun (the whole collection) rather than a place to navigate,
      and implies passive storage rather than an active, updating view.

---

## Episodes

- **Episode**: the atomic unit of content. A single audio file with associated
  metadata, belonging to a subscription.
  - See also: `DATA_MODEL.md` — Episode.

- **Playback status**: the user's current relationship with an episode.
  Valid values: **unplayed**, **in-progress**, **played**. Use these exact
  terms in code and user-facing copy; avoid synonyms like "listened",
  "finished", or "watched".

- **Flag** / **Starred**: a user-set marker on an episode. "Flag" is the
  internal term; the user-facing affordance may be a star or equivalent icon.
  Flagged episodes are protected from automatic eviction.

---

## Storage

- **Download**: the act of fetching an episode's audio file to local storage.
  Distinct from fetching feed metadata (which is a **refresh**).

- **Refresh**: fetching updated feed metadata from the server. Does not
  imply downloading episode files.

- **Eviction**: automatic removal of a downloaded audio file to free storage.
  Distinct from user-initiated deletion. Eviction never affects metadata.

---

## Settings

- **Cascading default**: a setting value inherited from a higher level in the
  hierarchy (global → subscription → episode) unless explicitly overridden at
  a lower level. See `DESIGN_PRINCIPLES.md` §3 for the full pattern.
  - In user-facing copy, cascading defaults are surfaced as:
    `[value] (default)` when inherited, `[value]` with a reset affordance
    when overridden.
