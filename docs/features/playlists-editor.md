# Playlist Editor
---
priority: high
depends: playlists, playlist-rules, subscription, library
---

The playlist editor is the interface for creating and editing playlists. It
combines a rules builder with a live preview of the resulting episode list.

## Layout

Two stacked components on mobile (each ~1/2 screen), columns on larger layouts.
A navbar provides Back/Cancel and Save actions.

```
NavBar: [Cancel] [Playlist Title]  [Save]
────────────────────────────────────────
Component 1: Rules & Settings  (~½)
────────────────────────────────────────
Component 2: Live Preview      (~½)
```

When canceling, warn user if unsaved changes.
Clicking the playlist title allows for renaming.

---

## Component 1: Rules & Settings

### Empty State

When no rules have been added:

```
Feeds
  (+) Add feed

Filters
  (+) Add filter

Sort
  (+) Add sort

▸ Settings  (collapsed)
```

### Feeds

The feeds section lists the sources for this playlist (subscriptions and/or
other playlists). Clicking "Add feed" opens the **Feed Selector** — a view
similar to the library with multi-select (checkboxes) across both subscriptions
and playlists. Selected feeds appear as rows in the feeds section.

### Filters

Each filter is a row:

```
[≡ drag] [include/exclude] [criteria type     ] [criteria value]
```

Example rows:
```
[≡] [include] [playback status] [unplayed      ]
[≡] [exclude] [duration       ] [> 2:00:00     ]
[≡] [include] [most recent    ] [2             ]
```

- **Drag handle** (≡): reorder filters by dragging
- **include/exclude toggle**: all filter types support this; user is responsible
  for sensible combinations
- **Criteria type**: selector for the filter operation (playback status,
  duration, most recent N, oldest N, etc.)
- **Criteria value**: parameter(s) for the selected operation

The "Add filter" button sits below all current filter rows and moves down as
rows are added.

**Filter order matters**: filters are applied top-to-bottom. Reordering changes
the result. The live preview reflects the current order.

Example: "most recent 2, then unplayed" gives the 2 most recent episodes if
unplayed. "unplayed, then most recent 2" gives the 2 most recent from the
unplayed set.

### Sort

Each sort is a row:

```
[≡ drag] [asc/desc] [sort key          ] [parameters  ]
```

Example rows:
```
[≡] [⬆] [in feeds ] [(feed list) →  ]
[≡] [⬆] [has flag  ] [(flag type) → ]
[≡] [⬇] [date      ] [              ]
```

- **asc/desc**: for boolean keys (e.g. "has flag"), desc = truthy first
- **sort key**: the field to sort on
- **parameters**: some sort keys open a sub-selector (e.g. "in feeds" and
  "has flag" open the Feed Selector and flag type picker respectively);
  "date" has no parameters

Sort rows are applied in order (primary → secondary → tertiary, etc.).

### Settings

A collapsed section at the bottom of Component 1, using the standard feed
settings interface (see `settings.md`). Expanded on tap. Not distracting in
the default collapsed state.

---

## Component 2: Live Preview

A scrollable episode list showing the result of the current rules as if the
playlist were saved now. Updates in real-time as rules are added, removed,
reordered, or changed.

The episode rows in the preview are the same format as in the full playlist
view (see `playlists.md`) — this is not a simplified list, it's the real thing.

If the rule set produces no episodes, show an empty state explaining why
(e.g. "No unplayed episodes match these filters").

---

## Feed Selector

A reusable view (also used for feed sort parameter selection) showing all
subscriptions and playlists with multi-select checkboxes. Structure matches
the library view (sections: Playlists, Shows, no suggestions). Previously selected items are
pre-checked on open. Feeds that are in the playlist sources are floated to the
top

---
