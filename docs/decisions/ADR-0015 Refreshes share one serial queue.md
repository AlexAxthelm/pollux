---
type: decision
status: accepted
feature:
  - "[[subscription-refresh]]"
related: []
---
# Refreshes share one serial queue

## Context

Several triggers can ask for a refresh (pull-to-refresh, Refresh all, the app becoming active, a background wake-up); see the table in [[subscription-refresh]].

## Decision

All triggers feed one **serial** queue in the core (one feed fetched at a time,
mirroring the download queue). A feed already queued or in flight is not queued
again. Refresh state is deliberately separate from the library's
`loading`/`error`, which the subscribe flow reads to detect success.

If `RefreshStale` arrives before the library has loaded (cold launch), the core
holds the request and honours it when the subscriptions arrive. If that load
failed (a storage error at launch), nothing else would ever retry it, so the next
`RefreshStale` retries the library load itself (one at a time, on every
activation until it succeeds), and the held request runs once it does. A
background run that finds the load failed simply does nothing; the retry happens
on the next foreground activation.

## Consequences

- One feed is fetched at a time, which is why a hung host matters; see [[ADR-0018 Feed requests time out after 20 seconds of silence]].
