---
type: feature
status: draft
priority: MVP
area: episode
parent: "[[episode]]"
depends:
  - "[[episode]]"
---
# Episode UI

How an episode is shown and acted on, in a list and on its own page. The state an
episode carries (playback status, download status, flags) and the context menu are in
[[episode]].

## List view (single row)

Displayed when episodes appear in a playlist, subscription, or search results:

- Small episode art
- Play button
- Title
- Three-dots menu
- Played status indicator
- Download status indicator (progress bar while downloading)
- Duration
- Playback position (if in-progress)

### Flags and actions in the list

Flags are visible on the row, and common episode actions can be reached from the list
without opening the episode: as swipe gestures where available, and as visible
actions when the user taps an episode, so the user can get to them without swiping.

### What a list shows

- The user can set which details are visible about episodes (information density).
- A show's backlog (previous episodes) can be viewed.
- Episodes removed from the feed are hidden by default and can be shown; this is not
  built yet. See [[subscription-delisted-episodes]].

## Swipe gestures (on episode list row)

Swipe actions are user-configurable. Defaults:

| Gesture | Default Action |
|---|---|
| Short swipe left | Flag / Unflag |
| Short swipe right | Download / Delete |
| Long swipe left | Played / Unplayed |
| Long swipe right | *(unassigned)* |

Each action is a toggle based on current state.

## Detail view (full page)

Expanded view for a single episode:

- Album art
- Title, feed name
- Release date, duration
- Show notes
- Chapters (if available)
- Chapter art (if available)
- Bookmark list (see [[bookmarks]])

Details of an episode that is not downloaded can be viewed before downloading it.

## Open questions

- Which details can the user show or hide on a row, and what are the density options?

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
