---
type: reference
---
# Implementation Plan

This is a rough ordering for MVP implementation, based on feature dependencies
and what needs to exist before other things can be built or tested meaningfully.

## Phase 0 — Foundation

Nothing else can start without this. Minimal UI elements.

- **Database schema**: episodes, subscriptions, playlists, play history, settings
- **RSS feed parsing**: fetch a URI, parse RSS 2.0, store feed + episode metadata
- **Feed refresh**: polling mechanism, HTTP conditional GETs (304 handling),
  background refresh where platform supports it. *Built:* manual (details page,
  Refresh all), foreground auto-refresh, best-effort `BGAppRefreshTask`, 304/429
  handling, persisted failure state. *Remaining:* refresh-interval settings,
  hiding removed episodes, the foreground-only notice. See
  [[subscription]] and [[subscription-refresh]].
- **Download manager**: queue, serial downloads for MVP, storage limit check
  before download, fail-on-full behavior (see [[downloads]] and [[storage-limits]])
- **Navigation structure**: Library (home), Settings entry point,
  Downloads entry point (see [[navigation]], where the Settings entry point is an
  open question)
- **Library view**: subscriptions list, empty state, alphabetical ordering (see
  [[library]] and [[library-ordering]])
- **Settings screen**: basic structure
- **Subscription preview + subscribe flow**: fetch URI, subscribe button (see
  [[subscription-subscribe]])
- **Subscription details page**: episode list


Milestone: can subscribe to a feed, see its episodes in the database, and
download one.

---

## Phase 1 — Color System

Do this before any major UI, so nothing needs to be retrofitted later.

- **base16 theming infrastructure**: all color references use token names,
  never hardcoded hex
  - #needs-decision Views paint with semantic roles, never tokens
    ([[ADR-0031 Views paint with semantic roles, not base16 tokens or hex]]), so this line
    and the code disagree. Kept as written; see the open question in [[theme]].
- **Built-in themes**: System (light/dark), Solarized (light/dark), Nord
- **Theme selector UI**: light/dark/follow radio, instant preview, save/cancel
  (see [[theme-ui]])

*Status: the infrastructure and the built-in themes are built. There is no Appearance
section yet, so the theme is set only through `Event::SetTheme`. See [[theme]].*

Milestone: app renders in multiple themes, theming is wired through from the
start.

---

## Phase 2 — Core UI Shell

Enough to navigate the app and see real data.

- **Library view**: sticky section headers
- **Settings screen**: global defaults
- **Subscription preview + subscribe flow**: subscribe button, redirect to
  details on subscribe
- **Subscription details page**: subscription settings stub, unsubscribe (hard
  delete for MVP)

Milestone: can subscribe to a podcast, see its episodes, and navigate the app.

---

## Phase 3 — Episode + Playback

The core listening experience.

- **Episode list view**: list row with art, title, play button, status
  indicators, three-dots menu, swipe gestures (see [[episode-ui]])
- **Audio playback engine**: OS media integration, position tracking, resume
  with rewind
- **Player UI**: artwork carousel (art page and show notes page),
  scrubber, skip buttons (30s/15s defaults), play/pause, source context,
  player options menu stub
- **Mini-player**: visible when active, play/pause + tap to expand, progress
  strip, disappears when nothing active
- **Active playlist persistence**: restore on cold start

*Status: the engine, player, mini-player and cold-start restore are built
(subscription-as-playlist only, no auto-advance; see
[[ADR-0006 No auto-advance until playlists exist]]). See [[player]], [[mini-player]] and
the ADRs in `docs/decisions/` (ADR-0001 to ADR-0013).*

Milestone: can find an episode, play it, control it from lock screen, resume
where you left off.

---

## Phase 4 — Playlists

The core differentiator.

- **Playlist data model**: query model (sources → filters → sorts), position
  tracking, active queue management (see [[playlists]] and
  [[ADR-0027 A playlist is a query over sources]])
- **MVP filter operations**: limit by recency (newest/oldest N), playback status, Deduplicate (on by default), title text match
- **MVP sort operations**: by age (asc/desc), by source (sequential)
- **Playlist view**: full view with summary stats, compact view
- **Playlist editor**: rules builder (feeds, filters, sorts), live preview,
  feed selector
- **Convenience playlists**: All Subscriptions, Downloaded/Available (see
  [[playlists-convenience]])
- **Circular dependency cap**: per-playlist episode limit (default 1,000), message if reached (see
  [[ADR-0030 Playlist resolution is capped by a per-playlist episode limit instead of detecting cycles]])

Milestone: can create a playlist with rules, play from it, and have it behave
as the active queue.

---

## Phase 5 — Polish + Remaining MVP

Rounding out MVP to a complete, shippable state.

- **Episode chapters** ([[episode-chapters]]): detect and parse (ID3, QuickTime, Podcasting 2.0),
  chapter navigation in player, chapter title in player title area
- **Storage UI**: usage breakdown by subscription, limit configuration (see
  [[storage-ui]])
- **Subscription unsubscribe**: warning with episode count, hard delete,
  playlist re-evaluation on removal; the player must stop and clear its saved play
  context if its episode's show is removed (see [[subscription-unsubscribe]],
  "Effect on playback")
- **Download queue management** ([[downloads-queue]]): downloads page, user re-ordering, and a
  persisted enqueue order so the serial queue restores FIFO across relaunch
  (today it rebuilds by pub date); groundwork for parallel downloads
- **Subscription-level download rules**: cascading defaults (global → feed); see
  [[downloads-auto]] and [[settings-cascade]]
- **Settings completion**: all known settings wired up, cascading defaults
  pattern throughout (see [[settings]])
- **Error states**: feed refresh failures (indicator on subscription row; the
  row marker and details-page line are built),
  download failures (retry button), invalid URI on subscribe, storage full
  notification (see [[downloads-failures]] and [[storage-notifications]])
- **Empty states**: library with no subscriptions, playlist with no matching
  episodes

Milestone: MVP complete. App is usable end-to-end for its core purpose.

---

## Post-MVP — v1.0

The archive binge feature is the reason this app exists. Once MVP is solid:

- **Show notes**: timestamp detection → synthetic chapters (the player's show-notes
  page itself is built; see [[player-show-notes]])
- **Archive binge playlist**: composite playlist template, surfaced in create
  playlist flow, one-tap setup (see [[playlists-archive]])
- **OPML import/export**: standard import for migrating from other apps (see
  [[import-export-opml]])
- **Playlist reordering**: behind mode toggle, alphabetical + by last updated (see
  [[playlists-playback]])

---

## Notes

- Platform target should be decided before Phase 2 (mobile-first is assumed,
  but affects implementation of background refresh, OS media integration, etc.)
  *Update: the app targets iOS today; see [[ARCHITECTURE]].*
- The rules engine (Phase 4) is the most architecturally significant piece —
  worth taking extra time to get the data model right before building UI on top
- Colors (Phase 1) is deliberately early; the cost of doing it late is high
- Sync is explicitly out of scope for MVP and v1.0 (see [[sync]])
