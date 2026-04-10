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

This should persist settings / metadata, but content files become unprotected,
and are eligible for immediate eviction

A "Delete now" button is available for users who want immediate hard deletion.
"Delete now" also immediately removes content files
