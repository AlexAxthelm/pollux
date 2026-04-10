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
