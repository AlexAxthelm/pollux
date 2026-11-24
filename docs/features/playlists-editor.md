---
type: feature
status: accepted
priority: high
area: playlists
parent: "[[playlists]]"
depends:
  - "[[playlists]]"
  - "[[playlists-rules]]"
  - "[[subscription]]"
  - "[[library]]"
---
# Playlist Editor

The playlist editor is the interface for creating and editing a playlist. It combines the
sources, a stack of rules and a live preview of the resulting episode list.

*Not built yet.* The layout below comes from the user's sketch (November 2026).

## Layout

One scrolling screen with four parts. Sources, Rules and Settings are collapsible; the preview
follows them.

```
[‹ Back]  Playlist name  [Save]  [↶ Undo] [↷ Redo]
────────────────────────────────────────────────
▾ Sources
   ≡ Playlist A
   ≡ Show B
   ≡ Playlist C
   ≡ 12 other episodes  (tap to edit)
  [+ Playlist] [+ Show] [+ Episode]

▾ Rules
   ≡ Deduplicate                    ⊖
   ≡ filter: unplayed               ⊖
   ≡ filter: duration < 2h          ⊖
   ≡ sort: oldest first             ⊖
  [+ Filter] [+ Sort]

▾ Settings                 [Advanced ◯]
   Episode limit 5k   (advanced)
   Play next: from top

Preview                    [⋯ details]
   Episode 1            [↑][↓]
     from A
   Episode 2            [↑][↓]
     from A
   Episode 3            [↑][↓]
     from B
   Episode 4            [↺][≡]   pinned to end
   Episode 5            [↺][≡]
```

## Top bar

- **Back** on the left. Leaving with unsaved changes warns first.
- Tapping the playlist name renames it.
- **Save** saves the playlist.
- **Undo** reverts the last editor action, repeatedly (a stack). Once something has been undone, **Redo**
  appears. See [[US-0245 Undo and redo in the playlist editor]].

## Sources

Each source is a row with a drag handle. Sources are ordered: the order is what the "by source" sort
and Deduplicate's "first" use ([[playlists-rules]]).

- **Playlists and shows:** added with **+ Playlist** and **+ Show**, each opening a multi-select
  picker (the Feed Selector below). If an added playlist already includes this one, the editor asks
  first ([[US-0244 Be asked before making a playlist loop]]).
- **Hand-picked episodes:** shown as one row ("12 other episodes") that is ordered among the other
  sources like any of them, with a hint that it can be tapped to edit. **+ Episode** (later) and the
  episode context menu's **Add to playlist** add an episode to it. This is the playlist's own hand-picked list
  ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]).

## Rules

One list of operations on the playlist, applied top to bottom: Deduplicate, filters and sorts, in any
order ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]], draft). Each
row has a drag handle and a remove button. **+ Filter** and **+ Sort** add a row at the bottom.

- **Deduplicate:** present and on for a new playlist, at the top. It takes no criteria.
- **Filters:** `include` or `exclude`, a criterion and a value ([[ADR-0043 Every filter can include or exclude]]).
  Example rows: `include · playback status · unplayed`, `exclude · duration · > 2:00:00`,
  `include · most recent · 2`.
- **Sorts:** a direction, a key and, for some keys, parameters. Each sort orders the list as it stands at
  that point, so the last sort is the main order and the ones above it break ties.
- **Manual ordering:** a sort row that appears by itself once the user has pinned episodes (see the
  preview), starts at the bottom of the stack so the pins hold, can be moved like the others, and can be removed
  once nothing is pinned.

**Order matters.** Deduplicate keeps the first occurrence in the order the list has at that point,
which is the result of the rules above it. Example: "most recent 2, then unplayed" gives the 2 most recent
episodes if unplayed; "unplayed, then most recent 2" gives the 2 most recent of the unplayed set.

## Settings

Collapsed by default, using the standard settings interface ([[settings-cascade]]). The **Advanced**
switch in its header shows the advanced settings, such as **Episode limit** (default 1,000, per
playlist; see [[ADR-0030 Playlist resolution is capped by a per-playlist episode limit instead of detecting cycles]]).
**Play next** ("from top" or in sequence) is the per-playlist setting from [[playlists-playback]].

## Preview

A scrollable episode list showing the result of the current rules as if the playlist were saved now.
It updates as sources and rules change. The rows are the same as in the full playlist view
([[playlists-ui]]), not a simplified list. If nothing matches, an empty state says why (for example
"No unplayed episodes match these filters").

- **Order buttons:** an unpinned row has up and down buttons that send the episode to the top or the
  end of the list. A pinned row has a drag handle to order it within its group, and a reset button
  that unpins it and returns it to its natural place. A group at the top only shows when something is
  pinned to the top. ([[US-0242 Order a playlist by hand with top and end pools]])
- **Details menu:** the preview has its own options. **Show source** adds the `from A` line to each
  row (optional, off or on as the user chooses). **Show excluded** includes episodes that the rules
  have removed, marked as excluded, with no reason shown
  ([[US-0243 See when a filter hides a hand-picked episode]]). See
  [[US-0246 Choose what the playlist preview shows]].
- **Limit message:** "Showing first N episodes." appears at the end only if the list is longer than
  the limit.

## Pickers

The picker behind **+ Playlist** and **+ Show** is a reusable view (also used for choosing the sources
of a "by source" sort) with multi-select checkboxes. It is laid out like the library view. Items already
in the playlist are pre-checked and floated to the top. **+ Episode** is for later; it will probably open a search or a library-like view for finding episodes. It is the same action as **Add to playlist** in an episode's context menu ([[episode]]), reached from the other side; the context menu is built first.

---

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
