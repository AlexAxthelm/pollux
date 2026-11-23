---
type: feature
status: draft
priority: MVP
area: subscription
parent: "[[subscription]]"
depends:
  - "[[subscription]]"
---
# Unsubscribing

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

## Effect on playback

There is no unsubscribe flow yet (see ROADMAP Phase 5). When it is built, the player has to
handle its episode's show disappearing: the saved play context is deleted with the episode
(`ON DELETE CASCADE`), but the in-memory player would carry on, "From:" would navigate to a
subscription that no longer exists, and a context save would fail (it is capped at 5
attempts). The player should stop and clear when its episode's subscription is removed.
(User story: [[US-0161 Stop on unsubscribe]].)

## Effect on settings

A subscription has its own saved settings ([[settings-cascade]], [[ADR-0045 Settings are saved as one JSON document per level]]).
*As built:* the shell removes them in the same transaction as the subscription (`DeleteSubscription`).
*When unsubscribe is built:* the core also keeps each subscription's settings in memory
(`Model::subscription_settings`), and nothing removes that entry yet. The unsubscribe flow has to
drop it, or a later change for the same id would save a document for a subscription that no longer
exists, and the player could resolve an episode against settings of a show that is gone.

Two guards belong with that flow, because `SetSubscriptionSetting` doesn't check that the id is
still subscribed:

- A save that is still in flight when the subscription is removed, or any later change for the
  removed id, would re-create the settings row after the shell deleted it. The row would then be
  loaded back into `subscription_settings` at every launch, invisible and never deleted. The
  core should ignore a change for an id that isn't subscribed (the settings page can only be
  open for a subscription that exists).
- As a backstop, settings saved for an id with no matching subscription can be dropped when the
  settings are loaded, once the subscriptions are known. That also cleans up rows left by any
  other path.

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
