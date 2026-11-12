---
type: decision
status: accepted
feature:
  - "[[subscription-refresh]]"
related: []
---
# A failed episode-list reload after a refresh is a banner, not a refresh error

## Context

After a successful refresh, the explicit reload of the open feed's episode list (see [[ADR-0014 Selecting the feed on screen does not reload its episode list]]) can itself fail.

## Decision

**Failure surfacing.** The last error is persisted, shown as a warning marker on
the library row (the reason is read out by VoiceOver) and as a line under the
title on the details page. The next success clears it.

A separate failure can follow a successful refresh: the explicit reload of the open
feed's episode list (see the implementation note above) can itself fail. The
refresh did succeed, so that is not recorded as a refresh error. If a list is
already showing, the core keeps it and shows a non-blocking "Couldn't update the
episode list" banner (`SubscriptionDetailView::list_notice`) instead of replacing
the list with an error screen. Only when nothing is on screen yet (the initial
load) does a failed load become the blocking error.

The banner means the rows on screen are older than what is stored, so something has
to retry the reload, and two things do, both in place (the rows stay visible, no
spinner): **re-entering the feed**, and **the next successful refresh of it**, even a
304, which would otherwise leave the list alone because "nothing changed" on the
server says nothing about what the screen is showing. A failed refresh doesn't
retry, since it says nothing about whether the stored episodes are current. The
banner clears when a reload succeeds, and on switching to another feed.

## Consequences

- The rows on screen can be older than what is stored until a reload succeeds.
