# Feed parsing
---
priority: low
---

Beyond RSS (2.0), the app should suppor Atom and JSONFeed files as sources for
subscriptions. If there are other formats in common use, they should also be
supported. In general, this should be an internal expansion of feed parsing
capabilities, not a User-facing change (except that more sources will be
available to be subscribed to)

Similarly, Podcasting 2,0 is interesting, but not an MVP feature (or rather
collection of features) https://podcasting2.org/

for feeds with a Permanent redirect, the original uri should be recorded as
outdated as part of the feed metadata (so that future imports/subscription flows
don't dupliucate), and the new uri as the "working" one.

## Current state

Only RSS 2.0 is actually parsed. That is the intended MVP scope, but the way it
currently fails for other formats is worth knowing before this is picked up.

**Atom feeds subscribe successfully and produce zero episodes.** `parse_feed`
reads enclosures from `entry.media`, and `feed-rs` puts Atom's
`<link rel="enclosure">` in `entry.links` instead, so every entry is dropped by
the `entry.media.into_iter().next()?` filter. The channel-level metadata parses
fine, so the subscription is created, appears in the library, and looks healthy
— it just has no episodes, and nothing reports why. Adding Atom support is
mostly a matter of falling back to `entry.links` when `entry.media` is empty.

The same silent-empty behaviour should be expected for any format whose
enclosures do not land in `entry.media`. Whatever surfaces "this feed yielded no
episodes" to the user is worth building alongside the first format expansion,
rather than after.

Episode identity across refreshes is a related open problem and is tracked in
`DATA_MODEL.md` under Episode → Identity across refreshes.
