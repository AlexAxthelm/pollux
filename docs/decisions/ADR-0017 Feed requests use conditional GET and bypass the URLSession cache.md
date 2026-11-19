---
type: decision
status: accepted
feature:
  - "[[subscription-refresh]]"
related: []
---
# Feed requests use conditional GET and bypass the URLSession cache

## Context

Refresh should be informed by server-side HTTP conditional GETs, and in general play well with 200/304/429 HTTP codes.

## Decision

**Conditional GET.** The last successful response's `ETag` and `Last-Modified`
are stored on the subscription and replayed as `If-None-Match` /
`If-Modified-Since`. The shell bypasses URLSession's own cache for feed fetches,
otherwise a locally cached 200 could hide the 304.

## Consequences

- A 304 still counts as a successful refresh; see the outcomes table in [[subscription-refresh]].
