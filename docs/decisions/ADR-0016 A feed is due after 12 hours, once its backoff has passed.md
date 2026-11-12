---
type: decision
status: accepted
feature:
  - "[[subscription-refresh]]"
related: []
---
# A feed is due after 12 hours, once its backoff has passed

## Context

Automatic refresh (on activation and from the background) should fetch only the feeds that need it. The interval is 12 hours by default; per-feed and global settings are not built yet (see "Not yet" in [[subscription-refresh]]).

## Decision

**When a feed is "due"** (automatic triggers only): it has never been refreshed,
or `last_refreshed` is 12 or more hours ago (`REFRESH_INTERVAL_HOURS`), **and**
its `retry_after_until` has passed. Manual refresh (details page, Refresh all)
ignores the backoff: an explicit request wins.

Both timestamps are wall-clock values written earlier, so the check tolerates
them being wrong. A `last_refreshed` **in the future** (the clock was ahead, or
has since been corrected) counts as stale, since otherwise the feed would stay
not-due until real time reached it; the redundant fetch is a cheap conditional
GET and stamps a correct time. A `retry_after_until` **further ahead than the
24 hour cap** can only be skew (nothing is ever written that far out) and is
ignored rather than honoured. A small clock step back therefore costs at most one
extra conditional GET.

## Consequences

- The interval is the hardcoded `REFRESH_INTERVAL_HOURS` until Settings exists.
- Backoff itself is described in [[ADR-0019 Refresh failures back off by cause]].
