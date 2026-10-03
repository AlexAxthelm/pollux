# Feed Subscription & Library
---
priority: MVP
depends: episode, storage, settings
---

A subscription is a mapping between the app and an RSS/Atom/JSONFeed URI. It is
the primary way users add content to Pollux.

## Terminology Note (Internal)

- **Subscription**: a podcast/show, represented by a feed URI that the app polls
  for updates. 1:1 with an RSS/Atom/JSONFeed file.
- **Feed**: the general abstraction for "an ordered, potentially changing set of
  episodes." Both subscriptions and playlists satisfy this interface. The rest of
  the app talks to feeds, not specifically to subscriptions or playlists. 

These terms are internal. User-facing language should use "podcast" or "show"
for subscriptions, and "playlist" for user-created feeds.

There are no limits on the number of subscriptions (though in practice space
limitations may impose one). Internal note: the metadata database should have
higher priority in storage than content files

## Subscribing

The user provides a URI. The app fetches the feed file and presents a
**subscription preview**:

- Feed title, artwork, description
- Episode list (most recent first)
- Prominent "Subscribe" button

If the URI is already subscribed, show a warning and offer a button to navigate
to the existing subscription's details page. Optionally surface archive playlist
suggestions for re-listening.

On subscribe, the app stores feed metadata and begins the subscription machinery
(refresh schedule, download rules).

MVP is to support RSS files, for future enhancements, see Feed parsing spec

After hitting the "subscribe button" the user is taken to the subscription
details page, which they shouldn't notice, since the only difference between the
preview and the details page is the presence of the "subscribe" button, which
disappears when subscribing

### Authenticated Feeds

Feeds requiring authentication (e.g. Patreon, supporting-cast) are out of scope
for MVP. See `subscription-auth.md`.

## Feed Refresh

The app periodically fetches the feed URI to discover new episodes. Refresh
is a metadata-only operation — it does not trigger downloads directly. Download
decisions are evaluated separately against the feed's download rules after a
refresh, but new metadata should trigger re-evaluation of those rules

### Refresh Schedule

A cascading default pattern applies: each subscription may set its own refresh
interval, falling back to the global default if unset.

Display pattern:
- Global default unset: `Refresh every 12 hours (default)`
- Global default changed: `Refresh every 24 hours (default)`
- Per-feed override: `Refresh every 1 hour (custom)`

Refresh can also be triggered manually from the subscription details page or
the library view (refresh all).

> **Implementation note (details page):** the core's `SelectSubscription` is
> idempotent — re-selecting the feed already shown does not reload its episode
> list (this is what keeps back-navigation from an episode detail from flashing
> a spinner). Consequently, when a refresh (or any path) writes new/updated
> episodes for the feed currently on screen, it must trigger an **explicit**
> reload of the details page's episode list; the page will not pick up storage
> changes on its own. Refresh does this in `RefreshSaved`, reloading without
> setting `detail_loading` so the existing list stays on screen. See
> `SelectSubscription` / `EpisodesLoaded` / `RefreshSaved` in
> `shared/src/app.rs`.

Feed refresh interval: 12 is the default, managed by settings (cascading)

Refresh should be informed by server-side HTTP conditional GETs, and in general
play well with 200/304/429 HTTP codes

If the OS/platform supports background operations, then
polling/refresh/update/downloads
should happen in background. If not, then there should be a signal to user
(panel somewhere?) that the app only updates when it's foregrounded.

### Refresh as built

*Status: implemented, except where listed under "Not yet".*

**Triggers**

| Trigger | Entry point | Scope |
|---|---|---|
| Pull-to-refresh on the details page | `RefreshSubscription(id)` | that feed |
| "Refresh all" toolbar button, or pull-to-refresh on the library list | `RefreshAll` | every feed, in library (title) order |
| App becomes active, including cold launch | `RefreshStale` | feeds that are due |
| Background wake-up (`BGAppRefreshTask`) | `RefreshStale` | feeds that are due |

All triggers feed one **serial** queue in the core (one feed fetched at a time,
mirroring the download queue). A feed already queued or in flight is not queued
again. Refresh state is deliberately separate from the library's
`loading`/`error`, which the subscribe flow reads to detect success.

**When a feed is "due"** (automatic triggers only): it has never been refreshed,
or `last_refreshed` is 12 or more hours ago (`REFRESH_INTERVAL_HOURS`), **and**
its `retry_after_until` has passed. Manual refresh (details page, Refresh all)
ignores the backoff: an explicit request wins.

If `RefreshStale` arrives before the library has loaded (cold launch), the core
holds the request and honours it when the subscriptions arrive.

**Conditional GET.** The last successful response's `ETag` and `Last-Modified`
are stored on the subscription and replayed as `If-None-Match` /
`If-Modified-Since`. The shell bypasses URLSession's own cache for feed fetches,
otherwise a locally cached 200 could hide the 304.

**Outcomes**

| Result | Effect |
|---|---|
| 200 | Parse, upsert (see `DATA_MODEL.md`), store new validators, clear the error and backoff, reload the open feed's episodes |
| 304 | Only `last_refreshed` moves; error and backoff cleared; episodes untouched |
| 429 | `retry_after_until` = now + `Retry-After` (seconds or HTTP-date, normalized to seconds by the shell), or 1 hour (`RATE_LIMIT_BACKOFF_SECS`) if absent |
| Other status, network error, unparseable body, failed save | `last_refresh_error` recorded; `retry_after_until` = now + 15 minutes (`FAILURE_BACKOFF_SECS`) so a broken feed isn't retried on every foreground |

A failed refresh never moves `last_refreshed` and never discards the stored
validators.

**Failure surfacing.** The last error is persisted, shown as a warning marker on
the library row (the reason is read out by VoiceOver) and as a line under the
title on the details page. The next success clears it.

**Background refresh.** `BGAppRefreshTask` is best-effort: iOS decides when, and
whether, it runs, and the request time is only a lower bound. It is requested for
**3 hours** out each time the app backgrounds and at the start of every run (so
the chain survives a run cut short). That is deliberately shorter than the 12-hour
interval: only due feeds are fetched, so an early wake is cheap, whereas a longer
request would push the system's real run well past the interval. A run gets about
30 seconds, so it refreshes as many due feeds as fit; the rest wait for the next
foreground or wake-up. When the system expires the task, the shell sends
`CancelRefresh`: the feeds still waiting are dropped (and any auto-refresh held
for the library load), while the fetch already in flight is left to finish and
have its outcome recorded, so nothing further starts. It does not download. It will not run if Background App
Refresh is off, in Low Power Mode, or after the user force-quits the app, so
foreground refresh is the reliable path. Declared in `iOS/project.yml`
(`UIBackgroundModes: fetch`, `BGTaskSchedulerPermittedIdentifiers`); the
identifier lives in `iOS/Pollux/BackgroundRefresh.swift` and a test checks the
two agree.

**Not yet**

- Per-feed and global refresh-interval settings. The interval is the hardcoded
  `REFRESH_INTERVAL_HOURS`; the cascading pattern waits on the Settings feature.
- Re-evaluating download rules after a refresh (no download rules exist yet).
- The "app only updates while foregrounded" signal when background refresh is
  unavailable (deferred to Settings).
- A per-feed "refreshing" indicator on the details page beyond the pull-to-refresh
  spinner; the empty state of a feed with no episodes cannot be pulled to refresh.

## De-listed Episodes

*Status: the marking is implemented; the hiding and toggle below are not.*

When an episode is no longer present in a feed's RSS/Atom/JSONFeed:

- Episode metadata is retained permanently in the local database
- Episode is hidden from the default subscription view
- A "Show unavailable episodes" toggle reveals them
- If the episode audio file is still on device, it remains playable
- Episode is marked with "Removed from feed" status (see `episode.md`)

As built, a refresh marks an episode `RemovedFromFeed` only when it is
`NotDownloaded` or `Failed`. A downloaded, queued, or in-flight episode keeps
its state so its file stays playable and its download is not orphaned. If a
marked episode reappears in the feed it returns to `NotDownloaded`. A refresh
that returns an **empty** feed marks nothing, since that is far more likely a
broken response than every episode being deleted. The episode list does not yet
hide removed episodes or offer the "Show unavailable episodes" toggle.

Permanent metadata retention is intentional — it supports the library model
and keeps the archive complete even as publishers rotate content.

## Unsubscribing

see `subscription-soft-delete.md` for details

for MVP: if a subscription is removed, all assosciated content files are also
deleted.

## Per-Feed Settings

The cascading default pattern applies to all per-feed settings. Each setting
shows whether it is using the global default or a custom value, and can be
reset to default individually.

Known per-feed settings:
- Refresh interval
- Download rules (see `storage.md`)
- *(more to be defined)*


## UI

### Library View

- List or grid of subscriptions (feed artwork)
- "Refresh all" trigger
- Link to add new subscription

### Subscription Details Page

- Feed artwork, title, description
- Episode list (full, including de-listed episodes behind toggle)
- Per-feed settings
- Unsubscribe (in menu, protected)

### Subscription Preview (pre-subscribe)

- Feed artwork, title, description
- Episode list preview
- Prominent "Subscribe" button
- If already subscribed: warning + "Go to subscription" button
