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
   Next episode: Play next

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

In the first version the Settings row holds only the minimal playlist settings, **Advanced** shows nothing until there are advanced settings, and **Next episode** has one choice because **From the top** is later. The sketch also has no Undo/Redo, no **+ Episode**, no hand-picked row, no pinned rows and no **⋯ details** menu; see the MVP slice in [[playlists]].

## Creating a playlist

A **+** in the library's Playlists section starts a new playlist ([[US-0196 Create playlist]]). It first offers starting points: **Blank**, or a template (later), with **Archive binge** as the first one ([[playlists-archive]]).

- **Blank** opens the editor with an empty name, no sources and Deduplicate at the top. Save is disabled until it is named.
- **A template** opens the same editor with its rules filled in as an ordinary, editable playlist and a blank sources list. It does not ask which shows to use. The first template is **Archive** (one playlist, [[playlists-archive]]); the composite with a Latest playlist is specified once the components are built.
- Duplicating a playlist, system ones included, is the other way to start from something that exists ([[US-0255 Duplicate a playlist]]).
- **MVP:** the **+** and Blank only. Templates come later.

## Top bar

- **Back** on the left. Leaving with unsaved changes warns first.
- Tapping the playlist name renames it.
- **Save** saves the playlist. It is disabled until the playlist has a name. A playlist with no sources, or with no rules, is valid: no sources is an empty playlist, no rules is source order. Edits take effect on Save; how they reach an episode that is playing is in [[ADR-0049 Playback moves on to whatever follows the finished episode in the re-resolved playlist]].
- **Undo** reverts the last editor action, repeatedly (a stack). Once something has been undone, **Redo**
  appears. See [[US-0245 Undo and redo in the playlist editor]].

## Sources

Each source is a row with a drag handle. Sources are ordered: the order is what the "by source" sort
and Deduplicate's "first" use ([[playlists-rules]]).

- **Playlists and shows:** added with **+ Playlist** and **+ Show**, each opening a multi-select
  picker (the Feed Selector below). The shows picker has an **All shows** item at the top, one source that follows the library
  ([[ADR-0050 A playlist with no sources is empty]]). A playlist with no sources is empty. If an added playlist already includes this one, the editor asks (later)
  first ([[US-0244 Be asked before making a playlist loop]]).
- **Hand-picked episodes:** shown as one row ("12 other episodes") that is ordered among the other
  sources like any of them, with a hint that it can be tapped to edit. **+ Episode** (later) and the
  episode context menu's **Add to playlist** add an episode to it. This is the playlist's own hand-picked list
  ([[ADR-0042 A playlist owns an ordered list of hand-picked episodes as a source]]).

## Rules

One list of operations on the playlist, applied top to bottom: Deduplicate, filters and sorts, in any
order ([[ADR-0047 A playlist's rules are one ordered stack of operations on the list]]). Each
row has a drag handle and a remove button. **+ Filter** and **+ Sort** add a row at the bottom.

- **Deduplicate:** present and on for a new playlist, at the top. It takes no criteria. There is at most one: **+ Filter** does not offer it while it is in the stack, and adding it back after removal puts it at the top.
- **Filters:** `include` or `exclude`, a criterion and a value ([[ADR-0043 Every filter can include or exclude]]).
  Example rows: `include · playback status · unplayed`, `exclude · duration · > 2:00:00`,
  `include · "first N episodes" · 20`.
- **Sorts:** a direction, a key and, for some keys, parameters. Each sort orders the list as it stands at
  that point, so the last sort is the main order and the ones above it break ties.
- **Manual ordering:** a sort row that appears by itself once the user has pinned episodes (see the
  preview). It can be moved and removed like the others, once nothing is pinned. When a sort is below it, the row says so ("Sorts below this will reorder pinned episodes"), and when First N episodes is below it, that pins apply before the cut. Fixed at the bottom is the fallback if this confuses ([[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]).

**Order matters.** Deduplicate keeps the first occurrence in the order the list has at that point,
which is the result of the rules above it. Example: "first 2, then unplayed" gives the first 2 episodes if unplayed;
"unplayed, then first 2" gives the first 2 of the unplayed set.

Two aids make the order readable (they are part of the MVP editor):

- **Count under each rule:** every rule row shows how many episodes are left after it ("→ 37 episodes"), so the effect of a rule and of moving it is visible. The full preview at any point in the stack is later ([[US-0247 Preview the rules at any point in the stack]]).
- **Helper text on the Rules section:** "Rules apply top to bottom. With several sorts, the last one is the main order and the ones above it break ties."

## Settings

Collapsed by default, using the standard settings interface ([[settings-cascade]]). Settings take effect when the playlist is saved; unsaved controls change only the editor's draft. The **Advanced**
switch in its header shows the less common settings, when there are any.
**Next episode** (**Play next** or **From the top**, the first unplayed episode) is the per-playlist setting from [[playlists-playback]].

## Preview

A scrollable episode list showing the result of the current rules as if the playlist were saved now.
It updates as sources and rules change. The rows are the same as in the full playlist view, except that there is no Play button (the preview is not playable)
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
- **Remove button:** a row from the hand-picked list can show a **Remove** button, which can be hidden ([[US-0252 Remove a hand-picked episode from the preview]]). Later, with the hand-picked list.
- **Work-bound message:** "This playlist was not fully loaded" appears at the end only if the work bound cut the resolution ([[ADR-0053 Resolution is bounded by the work it does, and playlists have no episode limit]]).

## Removing things other playlists depend on

*Later. The first version uses the ordinary delete and unsubscribe confirmations and drops the source from the playlists that use it.*

Deleting a playlist that other playlists include, and unsubscribing from a show that playlists use as a source, both confirm first with a message such as "This playlist is included in 5 other playlists", with a preview of those playlists ([[US-0251 See which playlists use a show or playlist before removing it]]). On confirm the source is dropped from each of them. A pin stays unless no other source still provides its episode ([[ADR-0051 Pinned episodes are a per-playlist list that the Manual ordering sort uses]]). "By source" needs nothing extra, because it reads the current Sources order.

## Pickers

The picker behind **+ Playlist** and **+ Show** is a reusable view with multi-select checkboxes. It is laid out like the library view. Items already
in the playlist are pre-checked and floated to the top. **+ Episode** is for later; it will probably open a search or a library-like view for finding episodes. It is the same action as **Add to playlist** in an episode's context menu ([[episode]]), reached from the other side. Hand-picked lists are not MVP, so neither entry point is.

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
