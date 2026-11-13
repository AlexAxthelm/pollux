---
type: decision
status: accepted
feature:
  - "[[subscription-refresh]]"
related: []
---
# Selecting the feed on screen does not reload its episode list

## Context

The core's `SelectSubscription` is idempotent: re-selecting the feed already shown does not reload its episode list. This is what keeps back-navigation from an episode detail from flashing a spinner.

## Decision

The core's `SelectSubscription` is
idempotent — re-selecting the feed already shown does not reload its episode
list (this is what keeps back-navigation from an episode detail from flashing
a spinner). Consequently, when a refresh (or any path) writes new/updated
episodes for the feed currently on screen, it must trigger an **explicit**
reload of the details page's episode list; the page will not pick up storage
changes on its own. Refresh does this in `RefreshSaved`, reloading without
setting `detail_loading` so the existing list stays on screen. See
`SelectSubscription` / `EpisodesLoaded` / `RefreshSaved` in
`shared/src/app.rs`.

## Consequences

- Any path that writes new or updated episodes for the feed currently on screen must trigger an explicit reload; the page will not pick up storage changes on its own.
- A failed explicit reload after a refresh is handled as described in [[ADR-0021 A failed episode-list reload after a refresh is a banner, not a refresh error]].
