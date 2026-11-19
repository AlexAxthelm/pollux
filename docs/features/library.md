---
type: feature
status: draft
priority: MVP
area: library
depends:
  - "[[subscription]]"
  - "[[playlists]]"
  - "[[navigation]]"
---
# Library

The library is the home screen of Pollux and the primary navigation hub. It
presents the user's subscriptions and playlists in a single scrollable view,
and is always accessible from anywhere in the app.

## Layout

A single vertical scrollable list, divided into sections. Each section has a
sticky header that remains visible as the user scrolls past it. Tapping a sticky
header jumps scroll back to the top of that section. Sections are not
collapsible — the list is continuous.

### Sections (top to bottom)

1. **Suggestions** *(not MVP — see [[library-recommendations]])*
   Horizontal scroll row or card grid of contextual recommendations.

2. **Playlists**
   User-created playlists, displayed as compact playlist rows (see
   [[playlists]] compact view). Includes system convenience playlists.

3. **Shows**
   All subscribed feeds, displayed as compact subscription rows (artwork,
   title, unplayed count). These are implicitly single-feed playlists and
   behave accordingly.

### MVP Initial State

With suggestions not yet implemented, the library opens to:

```
Playlists
  Playlist row
  Playlist row
  ...
Shows
  Show row
  Show row
  ...
```

As the user scrolls down past Playlists:

```
[Playlists]   ← sticky header, tappable to jump back
Shows
  Show row
  Show row
  ...
```

## Empty State

If there's no subscriptions, then we should have a prompt for the user to add subscription (MVP: simple text describing situation and where to find the button; "Add your first podcast, hit the + button below" or something like that)

## Ordering

Default ordering for both Playlists and Shows sections is **alphabetical**.
Further ordering options, consistency across screens, and locale-aware sorting are
in [[library-ordering]].

## Unplayed Count

Show rows display an unplayed episode count as text. User can configure
whether this shows:
- **Total unplayed** (default)
- **Immediately available unplayed** (downloaded only)

This is a global setting (see [[settings]]).

## Toolbar

The library toolbar contains:
- **Add subscription** button — primary entrypoint for subscribing to a new feed
- *(Later)* **Search / filter** input — find a specific show or playlist by name

If the toolbar is sparse, "Add subscription" placement may be revisited.

## Display Mode

Default is list view. Grid view is a separate feature (see [[library-grid]],
not MVP).

## Actions

- **Refresh all**: triggers a feed refresh for all subscriptions. Accessible
  via toolbar button. *As built:* the button (spinner while refreshing, disabled
  when the library is empty) and pull-to-refresh on the list both send
  `RefreshAll`, which refreshes feeds one at a time in list order. Each row
  shows a spinner while its feed is queued or fetching and a warning marker if
  its last refresh failed. See [[subscription-refresh]] for the triggers and
  outcomes.
- **Add subscription**: in toolbar. See [[subscription-subscribe]].
- **Downloads**: link to the download queue / storage screen, accessible from
  library (button in toolbar). See [[downloads-ui]] and [[storage-ui]].

For MVP, these should all be buttons in a toolbar

## Parts

- [[library-ordering]]: how the sections are ordered, including locale-aware sorting
- [[library-performance]]: scroll cost on long lists
- [[library-grid]]: the grid display mode (not MVP)
- [[library-recommendations]]: the Suggestions section (not MVP)

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
