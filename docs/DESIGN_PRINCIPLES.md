# Pollux Design Principles

This document captures the core design philosophy behind Pollux. It is intended
as a reference when designing new features or evaluating tradeoffs. It is not a
rigid ruleset, but as a guide to make implicit decisions explicit so that the app
stays coherent as it grows.

Read this alongside `docs/ARCHITECTURE.md` (technical) and `docs/ROADMAP.md`
(implementation order).

---

## 1. Multiple listening modes as first-class citizens

Most podcast apps are built around one implicit workflow: new episode arrives,
you listen, you move on. Pollux treats that as one valid mode among several:

- **Latest news** — keep up with a feed as it publishes
- **Revisiting an old friend** — Re-listen to favorite episodes from the past
- **Discovering something you missed** — dig into an archive, either
  systematically, or organically
- **Catching up after a break** — you've been away from a feed for a while and
  want to work through the backlog, but not necessarily all of it. Different
  from archive binge (which is deliberate and from the start) and from latest
  news (which assumes you're current).
- **Curated listening** — using playlists to mix episodes across feeds by theme,
  guest, topic. More of a DJ/editorial mode.

None of these modes is privileged. The app is designed so that each is as
well-supported and as easy to set up as the others.

The features that create psychological pressure in other apps — unread counts,
recency-first ordering, "X episodes behind" indicators — will exist in Pollux
too, because they are genuinely useful for some listening styles. But they are
not the frame the app imposes on everyone. Nudges toward library-style and
intentional listening are gentle and informational, not judgments.

**In practice:** when designing a new feature, ask whether it serves all modes
or only one. Features that would actively disadvantage some listening styles
need a strong justification.

---

## 2. Unified abstractions over special cases

The app deliberately avoids parallel systems for things that are fundamentally
the same concept.

A subscription is a playlist (a single-feed query with no filters). A
convenience playlist ("All Subscriptions", "Downloaded") is a playlist. An
archive binge template is a playlist. The playback engine, the rules engine,
and the UI only need to understand one primitive.

This means the rules system is intentionally general from the start. MVP scope
is limited by which filter and sort *operations* are implemented, not by
introducing a simpler model that would need to be replaced later.

**In practice:** when a new feature seems to need a new concept, first ask
whether it can be expressed as a configuration of an existing one. Introduce
new primitives only when the existing ones genuinely cannot express the
requirement.

---

## 3. Cascading defaults

Settings follow a consistent hierarchy: global → feed → episode (or global →
feed → feed → ... → subscription → episode, for nested playlist cases). Each level
inherits from the one above unless explicitly overridden.

The UI always makes the current state legible:
- `Refresh every 12 hours (default)` — inherited, global is at its own default
- `Refresh every 24 hours (default)` — inherited, global has been changed
- `Refresh every 1 hour` — local override; a reset affordance is shown

This pattern applies to: refresh interval, skip increments, download rules,
playback speed, and any other setting that makes sense to configure globally
but override locally.

**In practice:** new settings should almost always follow this pattern. A
setting that only exists at one level is the exception, not the rule. The UI
treatment (show current value, show whether it's default or overridden, offer
reset) should be consistent everywhere.

---

## 4. Inform before destruction, protect by default

The app never takes a destructive action silently. Content is not deleted
without warning; downloads do not fail silently; subscriptions are not removed
without surfacing consequences.

When a destructive action is about to occur, the default behavior is to stop
and tell the user what will happen — including specifics ("this will delete 27
episodes, including 3 saved ones"). The user must opt in to proceed.

This applies equally to automation. Auto-eviction, when it exists, is opt-in.
The user can see what would be evicted before it happens.

---

## 5. User is in absolute control, including of the protections

Protection defaults are there to prevent accidental data loss, not to
infantilize the user. Every protection can be reduced or removed by the user
if they choose.

- Warnings can be silenced ("don't warn me again")
- Destructive actions can be made easier (e.g. a long-press with haptic
  feedback instead of a confirmation dialog, if the user prefers)
- Automated systems (auto-eviction, download rules) can be delegated to — the
  user can choose to hand authority to the app rather than managing things
  manually

The goal is that power users never feel constrained, while new users are
protected from footguns they didn't see coming.

**In practice:** principles 4 and 5 are in tension and need to be held
together. A feature that only implements principle 4 (always warns, can never
be disabled) is restricting. A feature that only implements principle 5
(configurable, but the default is permissive) is dangerous. The defaults
protect; the controls allow the user to relax them.

---

## 6. Defaults should be good enough to ignore

The cascading defaults pattern (principle 3) only works if the defaults
themselves are well-chosen. A user who never visits settings should have a
good experience.

This is not a platitude — it has design consequences. When choosing a default,
prefer the option that is safer, less surprising, and appropriate for the
widest range of users. When a feature has a default that will be wrong for
many users, that's a signal the feature design needs more work.

---

## 7. Metadata is permanent; files are expendable (with exceptions)

Episode metadata — title, description, feed association, play history,
bookmarks, flags — is retained permanently, even when an episode is removed from
a feed's RSS or a subscription is unsubscribed. The local database is a complete
record of what the user has encountered. These records can be removed by user
action, but are persisted under normal operation

Audio files are a different matter. They are large, re-downloadable in most
cases, and subject to storage limits. They can be evicted or deleted; the
metadata remains.

However, not all audio files can be re-downloaded — episodes removed from a
feed may no longer be available on the server. Files in this situation, and
files the user has explicitly flagged/starred, are protected from automatic
eviction. The app distinguishes between "can be recovered" and "may be gone
forever" and treats them differently.

**In practice:** when designing storage or cleanup features, the question is
never "should we delete the metadata?" (probably not) but "can this file be
recovered if we delete it?" (sometimes yes, sometimes no — the app must find
out before acting).

---

## 8. Progressive enhancement from platform primitives

When the platform provides a standard capability that delivers a good user
experience — OS media controls, lock screen integration, system share sheets,
background download APIs, HTTP conditional GETs — use it. Don't reinvent it.

This applies both to features (use the system share sheet rather than building
our own) and to implementation (use HTTP 304 handling rather than re-fetching
full feeds unnecessarily).

Where platform capabilities are unavailable or inconsistent across targets, the
app degrades gracefully and informs the user (e.g. "this app only updates when
open" on platforms without background refresh), rather than silently providing
a worse experience.

**In practice:** before implementing something from scratch, check whether the
platform already does it. The bar for reinventing a platform primitive is high:
the platform version must be genuinely inadequate for the use case, not merely
customizable in ways we haven't explored yet.
