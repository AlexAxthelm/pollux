---
type: feature
status: accepted
priority: MVP
area: subscription
parent: "[[subscription]]"
depends:
  - "[[subscription]]"
  - "[[settings]]"
---
# Feed refresh

The app periodically fetches the feed URI to discover new episodes. Refresh
is a metadata-only operation — it does not trigger downloads directly. Download
decisions are evaluated separately against the feed's download rules after a
refresh, but new metadata should trigger re-evaluation of those rules

## Refresh schedule

A cascading default pattern applies: each subscription may set its own refresh
interval, falling back to the global default if unset.

Display pattern:
- Global default unset: `Refresh every 12 hours (default)`
- Global default changed: `Refresh every 24 hours (default)`
- Per-feed override: `Refresh every 1 hour (custom)`

Refresh can also be triggered manually from the subscription details page or
the library view (refresh all).

When a refresh writes new or updated episodes for the feed currently on screen, the
details page reloads its episode list explicitly; see
[[ADR-0014 Selecting the feed on screen does not reload its episode list]].

## Behavior

Feed refresh interval: 12 is the default, managed by settings (cascading)

Refresh should be informed by server-side HTTP conditional GETs, and in general
play well with 200/304/429 HTTP codes

If the OS/platform supports background operations, then
polling/refresh/update/downloads
should happen in background. If not, then there should be a signal to user
(panel somewhere?) that the app only updates when it's foregrounded.

## As built

*Status: implemented, except where listed under "Not yet".*

### Triggers

**Triggers**

| Trigger | Entry point | Scope |
|---|---|---|
| Pull-to-refresh on the details page | `RefreshSubscription(id)` | that feed |
| "Refresh all" toolbar button, or pull-to-refresh on the library list | `RefreshAll` | every feed, in library (title) order |
| App becomes active, including cold launch | `RefreshStale` | feeds that are due |
| Background wake-up (`BGAppRefreshTask`) | `RefreshStale` | feeds that are due |

All triggers feed one serial queue ([[ADR-0015 Refreshes share one serial queue]]). Which feeds are due, for the
automatic triggers, is decided as described in [[ADR-0016 A feed is due after 12 hours, once its backoff has passed]].

### Outcomes

**Outcomes**

| Result | Effect |
|---|---|
| 200 | Parse, carry over any title, artwork or description the response lacks (below), upsert (see `DATA_MODEL.md`), store new validators, clear the error and backoff, reload the open feed's episodes |
| 304 | `last_refreshed` moves; error and backoff cleared; no episode is rewritten. It also finishes pending removals: episodes already absent for 48 hours are flagged (see [[subscription-delisted-episodes]]), and the open list reloads if any were |
| 429 | `retry_after_until` = now + `Retry-After` (a delay in seconds or an HTTP-date in any of the three formats RFC 9110 requires; the shell forwards the header verbatim and the core parses it, so every platform shares one parser), or 1 hour (`RATE_LIMIT_BACKOFF_SECS`) if absent. Capped at 24 hours (`MAX_RETRY_AFTER_SECS`), so a host asking for a year (or a typo) can't silence auto-refresh for that feed indefinitely; a value too large to represent clamps to the cap rather than falling back to the 1 hour default |
| Device can't reach the network (`HttpResult::Unreachable`: offline, cellular data off, roaming off, on a call, connection lost mid-request) | `last_refresh_error` recorded; **no backoff** and any existing one is left as it was. Nothing is wrong with the feed, and backing it off would keep auto-refresh away for 15 minutes after connectivity returns, for every feed at once |
| Other status, host-side network error (refused, TLS, timeout, DNS), unparseable body, failed save | `last_refresh_error` recorded; `retry_after_until` = now + 15 minutes (`FAILURE_BACKOFF_SECS`) so a broken feed isn't retried on every foreground |

The reasoning for each is in the decisions below.

### Not yet

- Per-feed and global refresh-interval settings. The interval is the hardcoded
  `REFRESH_INTERVAL_HOURS`; the cascading pattern waits on the Settings feature.
- Re-evaluating download rules after a refresh (no download rules exist yet).
- The "app only updates while foregrounded" signal when background refresh is
  unavailable (deferred to Settings).
- A per-feed "refreshing" indicator on the details page beyond the pull-to-refresh
  spinner; the empty state of a feed with no episodes cannot be pulled to refresh.

## Decisions

- [[ADR-0014 Selecting the feed on screen does not reload its episode list]]
- [[ADR-0015 Refreshes share one serial queue]]
- [[ADR-0016 A feed is due after 12 hours, once its backoff has passed]]
- [[ADR-0017 Feed requests use conditional GET and bypass the URLSession cache]]
- [[ADR-0018 Feed requests time out after 20 seconds of silence]]
- [[ADR-0019 Refresh failures back off by cause]]
- [[ADR-0020 A degraded feed response does not erase stored metadata]]
- [[ADR-0021 A failed episode-list reload after a refresh is a banner, not a refresh error]]
- [[ADR-0022 Background refresh is requested every 3 hours and persists before it ends]]

## User stories

```base
views:
  - type: table
    name: User Stories
    filters:
      and:
        - file.hasLink(this.file)
        - type == "user-story"
    order:
      - file.name
      - status
    sort:
      - property: file.name
        direction: ASC
```
