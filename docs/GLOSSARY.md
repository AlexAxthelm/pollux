---
type: reference
---
# Pollux Glossary

This document defines terminology used in Pollux's codebase, documentation,
and user-facing copy. It is intended as both a dictionary and a style guide —
where a term has user-facing implications, those are called out explicitly.

Terms will be added as the data model and feature set are defined. If a term
is not in this glossary, that is a signal it needs to be discussed and added,
not that it can be used freely.

For code-facing definitions and entity relationships, see [[DATA_MODEL]].

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
  - See also: [[DATA_MODEL]] — Subscription.

- **Playlist**: a query over an ordered list of sources (possibly none), made by
  the user or provided by the app, with an ordered stack of rules (filters, sorts,
  Deduplicate) applied. Produces an ordered episode list.
  - Related terms:
    - **Smart playlist**: avoid this in user-facing copy. The word "playlist"
      covers all cases.
    - **Queue**: the user-facing term for the short "up next" list that plays
      before the active playlist resumes. It is not a playlist, and it is not the download queue ([[downloads-queue]])
      ([[ADR-0046 Queued episodes play before the active playlist resumes]],
      [[US-0152 Queue]]).
    - **Convenience playlist**: internal/doc term for system-provided playlists
      that require no configuration ("All subscriptions", "In progress"). They
      appear in the UI as playlists, but cannot be edited, renamed or deleted; the
      user duplicates one to change it ([[playlists-convenience]]).
    - **Duplicate**: the action that copies a playlist, system ones included, into a
      new playlist the user owns ([[US-0255 Duplicate a playlist]]).
  - See also: [[DATA_MODEL]] — Playlist, PlaylistSource, PlaylistRule,
    PlaylistPin.

- **Source**: where a playlist's episodes come from: a show, another playlist, the
  **All shows** source (every subscribed show, including ones added later), or the
  playlist's own **hand-picked list**. Sources are ordered; the order is what the
  "by source" sort uses ([[playlists-editor]]).
  - **Hand-picked list**: single episodes added to one playlist by hand ("Add to
    playlist"). It belongs to that playlist and works as one of its sources
    ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]).

- **Rule**: one step in a playlist's ordered stack of rules, applied top to bottom:
  a **filter** (keeps or drops episodes, with include or exclude), a **sort**, or
  **Deduplicate** ([[playlists-rules]],
  [[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]).
  - **First N episodes**: the filter that keeps the first N of the list at that
    point. It is the only way to cap a playlist's length.
  - **Manual ordering**: the sort that places pinned episodes first or last.

- **Pin**: an episode the user has sent to the top or the end of a playlist. A
  pinned episode stays where it was put while the **Manual ordering** rule is below
  the other sorts ([[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]).

- **EpisodeSource** *(placeholder — not a finalized term)*: the internal
  abstraction representing either a subscription or a playlist. Used in the
  playback engine, rules engine, and playlist editor, which need to operate on
  both without caring which kind they have. This term should not appear in
  user-facing copy. The right name for this concept will be settled when the
  data model is defined.
  - See also: [[DATA_MODEL]] — EpisodeSource.

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
  - See also: [[DATA_MODEL]] — Episode.

- **Playback status**: the user's current relationship with an episode.
  Valid values: **unplayed**, **in-progress**, **played**. Use these exact
  terms in code and user-facing copy; avoid synonyms like "listened",
  "finished", or "watched".
  - In a playlist filter the option is **Not played**: anything not marked played,
    so it includes started episodes ([[playlists-rules]]).

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
  hierarchy (global → subscription → playlist(s) → episode) unless explicitly
  overridden at a lower level. The chain an episode uses is the context it is played
  from; see [[ADR-0044 An episode's settings come from the context it is played from]]
  and [[DESIGN_PRINCIPLES]] §3 for the full pattern.
  - In user-facing copy, cascading defaults are surfaced as:
    `[value] (default)` when inherited, `[value]` with a reset affordance
    when overridden.
