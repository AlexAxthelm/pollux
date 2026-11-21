---
type: feature
status: accepted
priority: MVP
area: settings
parent: "[[settings]]"
depends:
  - "[[settings]]"
  - "[[subscription]]"
  - "[[playlists]]"
  - "[[episode]]"
---
# Settings cascade

How a setting's value is inherited from a broader level and overridden at a more
specific one. The catalog of global settings is in [[settings]].

## Cascading defaults pattern

Many settings follow a cascading default pattern: a global default applies
unless overridden at a more specific level (episode → playlist → subscription → global;
see Context stack below).

The UI should always make clear whether a value is using the global default or
a custom override, and offer a way to reset to default:

- `Refresh every 12 hours (default)` — using global default
- `Refresh every 24 hours (default)` — global default has been changed
- `Refresh every 1 hour` — per-subscription override is set; subtle reset button shown

This pattern applies to: refresh interval, download rules, skip increments, and
any other setting that makes sense to configure globally but override locally.

## Seeing what is customised

The user should be able to tell where a value comes from:

- A setting that differs from the app's own default is shown as adjusted.
- A setting that differs from the user's own global value is shown as overridden.
- A global setting the user has changed from the app default is shown as changed.

Changing a global setting is respected by everything that has not overridden it.
Returning a setting to its default should be easy, so the user doesn't need to know
what a sensible default is.

## Subscription settings

Subscription settings are reached from a **gear icon on the subscription's page**, which
opens a Subscription settings page. The page is clearly marked as editing that
subscription (its name is in the header), not the library or the global defaults. Some of
its settings are behind the **Advanced** flag (see [[settings-ui]]). See [[subscription]].

Sections inherit from the global page and use defaults, with a "(default)" note unless
changed. A value that isn't the default has a small "reset to default" button next to it.
- Playback
- Library (display options)
- Storage (how many episodes to keep locally)
- Refresh interval (subscriptions only)

Playlists have a settings page of the same shape, marked as editing that playlist; an unset
value there reads "Follows each source" (see Context stack).

*Superseded wording: an earlier version said these settings are "not from the main
settings screen" and, elsewhere, reachable "either from the main settings page, or by
clicking the gear". The gear on the subscription page is the decision. Whether the global
page also lists subscriptions is in [[settings]].*

## Context stack

*Accepted, see [[ADR-0044 An episode's settings come from the context it is played from]].*

An episode's settings come from the context it is played from. The most local set
value wins, walking up from the episode:

`episode → active playlist → included playlists on the route → subscription → global`

- Played from a subscription: episode → subscription → global.
- Played from a playlist A that includes B, where the episode comes via B:
  episode → A → B → the episode's subscription → global. A's own values apply to all of
  its episodes; where A leaves one unset, B's (or C's, for episodes via C) applies, so
  episodes from sibling sources in A can have different effective settings. Played from B
  instead, B is the most local.
- Played from the library: episode → subscription → global.
- An unset playlist value falls through to the episode's subscription, so a playlist's
  settings page shows **"Follows each source"** rather than a single inherited value.
- An episode override beats an explicit playlist value.
- Subscriptions and playlists take the same settings and look the same (US-0068).

### Which settings appear where

- Every setting is declared with the levels it applies to. Almost all apply to playlists
  as well as subscriptions; for example a playlist can raise storage to "keep all" before
  a trip while the show keeps one.
- Most settings are behind the **Advanced** filter (see [[settings-ui]]), and which
  settings the filter reveals depends on the level being edited.
- Playback-time settings are the ones the context stack is about. Storage, download and
  refresh settings can be set on playlists too, but have no playback context. A playlist is evaluated like a subscription: each can request that an episode be downloaded or retained, and storage
  combines the requests (default: retain). See [[storage-eviction]].

### Not decided yet

- Saving a speed or skip change made in the player to an episode or context is deferred.
  For now it applies to the player session only.

## Open questions

None

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
