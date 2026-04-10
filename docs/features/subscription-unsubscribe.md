# Unsubscribing
---
priority: MVP
depends: subscription
---

Unsubscribe is a destructive action and should be protected accordingly:
- Not in the primary UI — accessed via a menu or settings within the subscription
  details page
- Should warn about consequences (downloaded files, playlist membership, etc.)

When the user unsubscribes, the app should warn them that downloaded episodes
will be deleted (including those on other devices if known), espescially if any
have been flagged to be saved ("this will delete 27 episodes, including 3 saved
ones")

When a subscription is removed, playlists should have their rule re-evaluated,
so that the removal is reflected in those too.

see subscription-soft-delete.md for future unsubscribe improvements
