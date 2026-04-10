# Library
---
priority: MVP
---

The library is the home screen of Pollux and the primary navigation hub. It
presents the user's subscriptions and playlists in a single scrollable view,
and is always accessible from anywhere in the app.

## Layout

A single vertical scrollable list, divided into sections. Each section has a
sticky header that remains visible as the user scrolls past it. Tapping a sticky
header jumps scroll back to the top of that section. Sections are not
collapsible — the list is continuous.

### Sections (top to bottom)

1. **Suggestions** *(not MVP — see `library-recommendations.md`)*
   Horizontal scroll row or card grid of contextual recommendations.

2. **Playlists**
   User-created playlists, displayed as compact playlist rows (see
   `playlists.md` compact view). Includes system convenience playlists.

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

## Ordering

Default ordering for both Playlists and Shows sections is **alphabetical**.

*(Later)* Additional ordering options: by last updated, custom (drag to
reorder). Reordering is behind a mode toggle (e.g. three-dots menu →
"Reorder"), not inline drag by default.

## Unplayed Count

Show rows display an unplayed episode count as text. User can configure
whether this shows:
- **Total unplayed** (default)
- **Immediately available unplayed** (downloaded only)

This is a global setting (see `settings.md`).

## Toolbar

The library toolbar contains:
- **Add subscription** button — primary entrypoint for subscribing to a new feed
- *(Later)* **Search / filter** input — find a specific show or playlist by name

If the toolbar is sparse, "Add subscription" placement may be revisited.

## Display Mode

Default is list view. Grid view is a separate feature (see `library-grid.md`,
not MVP).

## Actions

- **Refresh all**: triggers a feed refresh for all subscriptions. Accessible
  via pull-to-refresh or toolbar button (TBD).
- **Add subscription**: in toolbar.
- **Downloads**: link to the download queue / storage screen, accessible from
  library (exact placement TBD).
