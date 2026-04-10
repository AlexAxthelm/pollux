# Circular dependencies in playlists
---
priority: high
---

By allowing the user to include other playlists as sources/feeds, the
possibility of circular dependencies is very real (a playlist includes itself as
a source, or more likely it includes another playlist that includes it, some
indirection to make it not obvious). I don't expect this to be a major problem,
as the subscriptions impose a finite set of episodes to draw from, which would
be further reduced with filters. In most situations, an episode will only appear
at its first position in the playlist, so later (circular) inclusions should be
a non-issue, but they may exist (perhaps intentionally for a never-ending
playlist). For playlists that may be unbounded the simple answer is probably to
only calculate the first N (1000? 10000?) items, and if there's something beyond
that, display a little message along the lines of "... and more" with a note
about the circular deps. Simirlarly, as a later feature, a circular dep warning
would be useful, but requires more thought on how to surface to the user in a
good way.

