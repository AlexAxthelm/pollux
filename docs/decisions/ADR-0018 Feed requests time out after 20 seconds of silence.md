---
type: decision
status: accepted
feature:
  - "[[subscription-refresh]]"
related: []
---
# Feed requests time out after 20 seconds of silence

## Context

Feeds are fetched one at a time ([[ADR-0015 Refreshes share one serial queue]]), and a background run gets only about 30 seconds.

## Decision

**Timeout.** A feed request fails after 20 seconds with no data
(`FeedFetcher.requestTimeout`). URLSession's 60 second default would let one hung
host stall every feed behind it in the serial queue, and use up a background
run's whole ~30 second budget. It is an idle timeout (it resets whenever data
arrives), so a slow but steady download of a large feed is unaffected; a host
that trickles bytes forever is not bounded by it. A timeout is a host-side error,
so the feed backs off. The subscribe flow uses the same fetch, so it gets the
same limit.

## Consequences

- A host that trickles bytes forever is not bounded by the idle timeout.
