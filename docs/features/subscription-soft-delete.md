# Soft Unsubscribe
---
priority: mid
depends: subscription, storage-eviction
---

## Summary

Unsubscribing is a soft action. The feed is marked inactive and metadata is
retained for 30 days before hard deletion. During the grace period, the user
can resubscribe and have everything restored as if the unsubscribe never
happened (play history, flags, playlist membership, etc).

On soft-delete, content files lose their protection — they are no longer
treated as belonging to an active subscription and become eligible for eviction.
For MVP (no auto-eviction), they are deleted with the subscription.
When eviction is implemented (see `storage-eviction.md`), these
files will be eviction candidates.

A "Delete now" button is available for users who want immediate hard deletion.
"Delete now" also immediately removes all associated content files.
