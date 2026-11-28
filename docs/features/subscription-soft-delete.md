---
type: feature
status: accepted
priority: mid
area: subscription
parent: "[[subscription]]"
depends:
  - "[[subscription]]"
  - "[[storage-eviction]]"
---
# Soft Unsubscribe

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

## Playlists, hand-picked episodes and pins

*Not decided; for when soft-delete is built.* The planned unsubscribe (not built yet) drops the show's hand-picked members, pins and source rows at once ([[subscription-unsubscribe]], [[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]], [[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]), because the MVP deletes the show with its episodes. With a grace period the summary above promises that "playlist membership" is restored, so one option is:

- During the 30 days the soft-deleted show counts as absent: its episodes are in no playlist (also not through the All shows source), and the hand-picked page and count skip its members.
- Its hand-picked members, pins and source rows stay stored without effect, and come back if the user resubscribes.
- At hard delete (30 days, or "Delete now") they are dropped, as today.
- The unsubscribe confirmation then says that playlists stop using the show now, and that it comes back if the user resubscribes in time.

Today's decisions do not stop this, but it is more than moving the drop to the hard delete. Pin cleanup removes a pin when no source provides its episode ([[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]), so opening a playlist during the grace period would remove pins that this promises to restore. Cleanup would have to leave alone the pins whose episodes are only temporarily unavailable because their show is soft-deleted.

## User stories

```base
views:
  - type: table
    name: User Stories
    filters:
      and:
        - file.hasLink(this.file)
        - type == "user-story"
    order:
      - file.name
      - status
    sort:
      - property: file.name
        direction: ASC
```
