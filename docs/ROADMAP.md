# Implementation Plan
---
This is a rough ordering for MVP implementation, based on feature dependencies
and what needs to exist before other things can be built or tested meaningfully.
---

## Phase 0 — Foundation

Nothing else can start without this. Minimal UI elements.

- **Database schema**: episodes, subscriptions, playlists, play history, settings
- **RSS feed parsing**: fetch a URI, parse RSS 2.0, store feed + episode metadata
- **Feed refresh**: polling mechanism, HTTP conditional GETs (304 handling),
  background refresh where platform supports it
- **Download manager**: queue, serial downloads for MVP, storage limit check
  before download, fail-on-full behavior
- **Navigation structure**: Library (home), Settings entry point,
  Downloads entry point
- **Library view**: subscriptions list, empty state, alphabetical ordering
- **Settings screen**: basic structure
- **Subscription preview + subscribe flow**: fetch URI, subscribe button
- **Subscription details page**: episode list


Milestone: can subscribe to a feed, see its episodes in the database, and
download one.

---

## Phase 1 — Color System

Do this before any major UI, so nothing needs to be retrofitted later.

- **base16 theming infrastructure**: all color references use token names,
  never hardcoded hex
- **Built-in themes**: System (light/dark), Solarized (light/dark), Nord
- **Theme selector UI**: light/dark/follow radio, instant preview, save/cancel

Milestone: app renders in multiple themes, theming is wired through from the
start.

---

## Phase 2 — Core UI Shell

Enough to navigate the app and see real data.

- **Library view**: sticky section headers
- **Settings screen**: global defaults
- **Subscription preview + subscribe flow**: subscribe button, redirect to
  details on subscribe
- **Subscription details page**: per-feed settings stub, unsubscribe (hard
  delete for MVP)

Milestone: can subscribe to a podcast, see its episodes, and navigate the app.

---

## Phase 3 — Episode + Playback

The core listening experience.

- **Episode list view**: list row with art, title, play button, status
  indicators, three-dots menu, swipe gestures
- **Audio playback engine**: OS media integration, position tracking, resume
  with rewind
- **Player UI**: artwork carousel (art page only, show notes placeholder),
  scrubber, skip buttons (30s/15s defaults), play/pause, source context,
  player options menu stub
- **Mini-player**: visible when active, play/pause + tap to expand, progress
  strip, disappears when nothing active
- **Active playlist persistence**: restore on cold start

Milestone: can find an episode, play it, control it from lock screen, resume
where you left off.

---

## Phase 4 — Playlists

The core differentiator.

- **Playlist data model**: query model (sources → filters → sorts), position
  tracking, active queue management
- **MVP filter operations**: limit by recency (newest/oldest N), playback status
- **MVP sort operations**: by age (asc/desc), by source (sequential)
- **Playlist view**: full view with summary stats, compact view
- **Playlist editor**: rules builder (feeds, filters, sorts), live preview,
  feed selector
- **Convenience playlists**: All Subscriptions, Downloaded/Available
- **Circular dependency cap**: 1,000 episode limit, message if reached

Milestone: can create a playlist with rules, play from it, and have it behave
as the active queue.

---

## Phase 5 — Polish + Remaining MVP

Rounding out MVP to a complete, shippable state.

- **Episode chapters**: detect and parse (ID3, QuickTime, Podcasting 2.0),
  chapter navigation in player, chapter title in player title area
- **Storage UI**: usage breakdown by subscription, limit configuration
- **Subscription unsubscribe**: warning with episode count, hard delete,
  playlist re-evaluation on removal
- **Subscription-level download rules**: cascading defaults (global → feed)
- **Settings completion**: all known settings wired up, cascading defaults
  pattern throughout
- **Episode identity**: stable `feed_guid` for feeds that omit `<guid>` — today
  every refresh re-inserts every episode and resets playback state. Release
  blocker; see `DATA_MODEL.md` → Episode → Identity across refreshes
- **Error states**: feed refresh failures (indicator on subscription row),
  download failures (retry button), invalid URI on subscribe, storage full
  notification
- **Empty states**: library with no subscriptions, playlist with no matching
  episodes

Milestone: MVP complete. App is usable end-to-end for its core purpose.

---

## Post-MVP — v1.0

The archive binge feature is the reason this app exists. Once MVP is solid:

- **Show notes**: player carousel page, timestamp detection → synthetic chapters
- **Archive binge playlist**: composite playlist template, surfaced in create
  playlist flow, one-tap setup
- **OPML import/export**: standard import for migrating from other apps
- **Playlist reordering**: behind mode toggle, alphabetical + by last updated

---

## Notes

- Platform target should be decided before Phase 2 (mobile-first is assumed,
  but affects implementation of background refresh, OS media integration, etc.)
- The rules engine (Phase 4) is the most architecturally significant piece —
  worth taking extra time to get the data model right before building UI on top
- Colors (Phase 1) is deliberately early; the cost of doing it late is high
- Sync is explicitly out of scope for MVP and v1.0
