---
type: decision
status: accepted
feature:
  - "[[subscription-refresh]]"
related: []
---
# Background refresh is requested every 3 hours and persists before it ends

## Context

`BGAppRefreshTask` is best-effort: iOS decides when, and whether, it runs. A run gets about 30 seconds, and iOS may suspend the app right after it ends.

## Decision

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
have its outcome recorded, so nothing further starts. A refresh counts as busy
until its outcome is **written** (`RefreshStatePersisted`), not merely decided:
the run ends once nothing is busy and iOS may suspend the app right after, so
finishing earlier could lose the timestamp, error or backoff. A failed write ends
the refresh like a successful one. It does not download, and
a background launch does not resume interrupted downloads either: the core only
loads those on `ResumePendingDownloads`, which the shell sends the first time
the app becomes active, never from `Started`. It will not run if Background App
Refresh is off, in Low Power Mode, or after the user force-quits the app, so
foreground refresh is the reliable path. Declared in `iOS/project.yml`
(`UIBackgroundModes: fetch`, `BGTaskSchedulerPermittedIdentifiers`); the
identifier lives in `iOS/Pollux/BackgroundRefresh.swift` and a test checks the
two agree.

## Consequences

- Foreground refresh is the reliable path.
- Background refresh is off when Background App Refresh is off, in Low Power Mode, or after a force-quit.
