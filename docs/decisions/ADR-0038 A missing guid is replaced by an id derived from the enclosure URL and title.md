---
type: decision
status: accepted
feature:
  - "[[feed-parsing]]"
related: []
---
# A missing guid is replaced by an id derived from the enclosure URL and title

## Context

`<guid>` is optional in RSS 2.0. When it is
missing, `feed-rs` would synthesise a random UUID that differs on every parse, so
the unique key never matched and every refresh re-inserted every episode — the
library grew without bound and playback state reset to Unplayed.

## Decision

Resolved for RSS: `parse_feed` registers `feed-rs`'s `id_generator` hook, which fires only for
entries with no id, to mark them; those get a locally derived `feed_guid` from
`stable_episode_id(enclosure_url, title)` instead. Entries with a real `<guid>`
are untouched.

Two properties of the derived id worth keeping in mind:

- It is **not** keyed on the parse-time `subscription_id` (a fresh UUID each
  parse — folding it in would reintroduce the instability). Storage already
  scopes uniqueness per feed, so the derived value only needs to be unique
  within one feed.
- It uses a hand-rolled FNV-1a rather than `std`'s `DefaultHasher`, whose
  algorithm `std` does not guarantee across releases; stored ids must stay valid
  across upgrades. A known-answer test locks the hash so it cannot silently
  drift.

## Consequences

Residual failure modes (accepted, and milder than every-refresh churn): editing
a title or a change to the enclosure URL yields a new id and thus a duplicate
row; two entries sharing *both* title and enclosure URL collapse to one. This
does not yet cover Atom, which is moot until Atom entries parse at all (see
[[feed-parsing]]).

The key that decides whether a refresh updates or inserts is described in [[DATA_MODEL]] (Episode → Identity across refreshes).
