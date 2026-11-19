---
type: decision
status: accepted
feature:
  - "[[subscription-refresh]]"
related: []
---
# Refresh failures back off by cause

## Context

Different failures say different things about the feed, so they are not all backed off the same way. The outcomes are tabulated in [[subscription-refresh]] (429, device unreachable, and other errors).

## Decision

The shell decides which `URLError` codes count as the device being offline
(`FeedFetcher.deviceConnectivityCodes`). Timeouts, DNS failures and refused
connections are deliberately **not** in that set, since they can't be told apart
from a dead host.

A failed refresh never moves `last_refreshed` and never discards the stored
validators.

## Consequences

- A small clock step back costs at most one extra conditional GET; see [[ADR-0016 A feed is due after 12 hours, once its backoff has passed]].
