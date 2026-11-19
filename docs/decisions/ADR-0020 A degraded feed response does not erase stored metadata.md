---
type: decision
status: accepted
feature:
  - "[[subscription-refresh]]"
related: []
---
# A degraded feed response does not erase stored metadata

## Context

A trimmed or half-edited feed response would otherwise rename a feed to its URL (reordering the library) and drop its artwork until the next refresh.

## Decision

**Metadata survives a degraded response.** The upsert writes the parsed title,
artwork and description as-is, and `parse_feed` yields "no value" for a feed
without them (the title falls back to the feed URL; artwork and description are
empty). "Without them" includes elements that are present but **blank**: `feed-rs`
hands `<title></title>`, `<description>   </description>` or a whitespace-only
image URL through as empty strings, so `parse_feed` trims every value and treats a
blank one as missing. That is the single place "absent" is decided, and it also
means a brand-new feed with an empty `<title>` is listed under its URL instead of
as a blank row.

Left alone, a trimmed or half-edited response would rename a feed to its URL
(reordering the library) and drop its artwork until the next refresh. So
`Subscription::inherit_missing_metadata` fills each of those three from the stored
subscription when the response lacks it, field by field, on both the refresh and
re-subscribe paths. A value the response does provide always wins, so a publisher
changing its title or artwork is still picked up. The trade-off is that a
publisher *removing* its artwork or description is not noticed; a stale image or
blurb is harmless and a lost one is not.

## Consequences

- A publisher *removing* its artwork or description is not noticed. A stale image or blurb is harmless and a lost one is not.
