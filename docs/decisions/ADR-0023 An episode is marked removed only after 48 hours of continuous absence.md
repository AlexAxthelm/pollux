---
type: decision
status: accepted
feature:
  - "[[subscription-delisted-episodes]]"
related: []
---
# An episode is marked removed only after 48 hours of continuous absence

## Context

A truncated or stale feed response (a CDN glitch, a feed briefly serving only its newest items) could mark most of a feed removed, and removed episodes cannot be downloaded until they reappear.

## Decision

As built, an episode is marked `RemovedFromFeed` only once it has been
**continuously absent for 48 hours of wall time**: the first refresh that doesn't
see it starts a clock, and an episode is flagged only if a refresh at least 48
hours later still doesn't see it. A single absence is not enough: a truncated or
stale response (a CDN glitch, a feed briefly serving only its newest items) would
otherwise mark most of a feed removed, and removed episodes cannot be downloaded
until they reappear. The window is time rather than a count of refreshes because
a count is reached in minutes: during one stale-cache incident a user pulling to
refresh a few times would see the same bad response each time. A feed that
genuinely dropped an episode is flagged by the first refresh after the window,
including a "latest episode only" feed, where every older episode goes after 48
hours.

"Refresh" here includes a 304. A feed that stops changing answers every later
refresh with one, and without help the episode a body first omitted would never be
flagged. So a 304 flags the episodes whose clock has already run 48 hours. It never
starts a clock (no body was compared, so nothing is known to be missing), and a
failed or rate-limited refresh doesn't flag anything, because neither says the
stored list still matches the server.

The clock restarts whenever the episode reappears: missing, present, missing is
two separate absences, not one long one. A clock recorded in the future (the wall
clock was ahead and has since been corrected) is restarted at the true time rather
than trusted.

Only `NotDownloaded` or `Failed` episodes are flagged. A downloaded, queued, or
in-flight episode keeps its state so its file stays playable and its download is
not orphaned; its clock keeps running, so if the file is later deleted the next
refresh flags it. If a marked episode reappears in the feed it returns to
`NotDownloaded`. A refresh that returns an **empty** feed marks nothing and does
not start a clock, since that is far more likely a broken response than every
episode being deleted. The episode list does not yet hide removed episodes or
offer the "Show unavailable episodes" toggle.

## Consequences

- A "latest episode only" feed loses every older episode to the flag after 48 hours.
- Hiding removed episodes and the "Show unavailable episodes" toggle are not built; see [[subscription-delisted-episodes]].
- Flagged episodes are protected from automatic cleanup; see [[storage-protection]].
