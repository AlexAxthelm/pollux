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

- `Refresh every 12 hours (default)` — on the global page, using the app default
- `Refresh every 24 hours (inherited from library)` — on a subscription's page, following the
  global value (whether or not that is the app default)
- `Refresh every 1 hour` — per-subscription override is set; subtle reset button shown

The global page says "(default)" because its value follows the app default. A subscription's
page says what the value follows, "(inherited from library)", since "default" there would hide
that the global value may have been changed.

This pattern applies to: refresh interval, download rules, skip increments, and
any other setting that makes sense to configure globally but override locally.

## Seeing what is customised

The user should be able to tell where a value comes from:

- A setting that differs from the app's own default is shown as adjusted.
- A setting that differs from the user's own global value is shown as overridden.
- A global setting the user has changed from the app default is shown as changed.

A setting counts as customised because the user **set** it, not because its value differs.
A value set to the same number as the current default still counts as set: it is shown as
customised, offers a reset, and stays where it is if the default later changes. That is what
makes "I chose this" different from "I never touched it" (see [[US-0181 Notifications for changes in default]]).
Resetting removes the setting, so it follows the default again.

Changing a global setting is respected by everything that has not overridden it.
Returning a setting to its default should be easy, so the user doesn't need to know
what a sensible default is.

## Subscription settings

Subscription settings are reached from a **gear icon on the subscription's page**, which
opens a Subscription settings page. The page is clearly marked as editing that
subscription (its name is in the header), not the library or the global defaults. Some of
its settings are behind the **Advanced** flag (see [[settings-ui]]). See [[subscription]].

Sections inherit from the global page, with an "(inherited from library)" note unless changed.
A value the subscription has set has a small reset button next to it, which returns it to the
global value.
- Playback
- Library (display options)
- Storage (how many episodes to keep locally)
- Refresh interval (on subscriptions and on playlists: a playlist's refresh interval is a request, and anything can request a refresh, as with storage and download settings, [[ADR-0044 An episode's settings come from the context it is played from]])

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

## As built

The global and subscription levels are built, in the core and in the app. The core has one
settings type shared by every level, where an unset value is `None`, and one resolver that takes
the levels that apply, most local first, and returns every setting with a concrete value. The
player calls it with the episode's own subscription's settings, then the global ones, so a
subscription that sets a skip or rewind beats the global value and one that leaves it unset
follows it. Adding a level (a playlist on the route) is adding a layer, not changing the
resolver. Resolution is not cached: it runs where a value is needed (a skip, a resume, starting
an episode) and when the player view and the settings pages are built, which is on every render,
position ticks included. It is a few comparisons, so that is fine; if it ever isn't, the resolved
values can be cached on the active playback. This is how it works today, with the two levels that are built. Once playlists exist, the route and the resolved values are taken when an episode starts and kept with the play context, and the player uses that copy, which re-resolves along the same route when a setting at a level on it is edited ([[ADR-0044 An episode's settings come from the context it is played from]]).

Each settings page shows, for each setting, the value in effect, what a reset would give, and
whether it is set:

- **Global page** (gear in the library toolbar): an unset setting reads "(default)", and a reset
  gives the app default.
- **Subscription page** (gear on the subscription's page, "Podcast settings"): an unset setting
  shows the global value it follows and reads "(inherited from library)", and a reset gives the
  global value. A change on the global page shows through at once.
- A value set to the same number as the one it would inherit still counts as set (see Seeing
  what is customised).

Not built yet: playlist and episode settings and their pages ("Follows each source"), walking the
route through included playlists, and the Advanced filter. See [[settings]] for the screens and
[[ADR-0045 Settings are saved as one JSON document per level]] for how settings are saved.

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
