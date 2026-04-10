# Audio Playback
---
priority: MVP
depends: episode, mini-player, subscription, settings
---

The player is the primary listening interface. It integrates with the OS media
system where available, and surfaces controls for the currently active episode
and playlist.

## OS Integration

- Use system media intents for playback control (lock screen, headphone buttons,
  car/speaker integration)
- Resume position on return: if the user leaves and comes back, playback resumes
  approximately where they left off. A small rewind on resume (e.g. 3-5s) is
  acceptable and common practice for context overlap. User can disable that
  behavior (see `settings.md`)

## Paged Content Area

The top portion of the player is a paged view (swipeable carousel with dot
indicators). MVP pages:

1. **Artwork**: episode image → feed image → placeholder. Chapter art shown
   if available and episode is in a chapter (see `episode-chapters.md`).
2. **Show notes**: placeholder page for MVP ("Show notes coming soon").
   See `player-show-notes.md` for full implementation.

*(Future)* Additional pages: chapter list, visualizer (see `visualizer.md`)

Note: for large screens, MVP will remain paged, but multiple pages may be shown
simultaneously if space allows.

## Scrubber / Position

A draggable progress bar showing current position in the episode.

- Current time shown on left, total time on right
- Tapping the total time toggles to remaining time
- *(Future)* Chapter markers overlaid on the scrubber

## Controls

### Skip Buttons

Forward and back skip buttons, configurable independently:

- **Default**: 30s forward, 15s back
- **Configurable**: global default, overridable per feed (cascades global → feed)
  (see `settings.md`)
- Button label displays the current increment (e.g. "+30" / "-15")
- Skipping past the end or beginning of the current file does not overshoot
  into the next/previous episode

### Chapter Navigation

Chapter back / chapter forward controls appear when the episode has chapters.
See `episode-chapters.md` for full behavior.

### Play / Pause

Standard toggle. Shows pause symbol when playing, play symbol when paused.

## Source Context

The player displays the active playlist/source context (e.g. "From: Music -
Boppy"). This is tappable and navigates back to the source playlist or
subscription view.

This also appears in the mini-player (see `mini-player.md`).

## Player Options Menu

Accessible from the player UI. MVP contents:
- OS output/routing options (where available)
- *(placeholder: speed control — see `speed-control.md`)*
- *(placeholder: sleep timer — see `sleep-timer.md`)*
- *(placeholder: equalizer — see `equalizer.md`)*

## Episode Options Menu

Standard three-dots episode context menu (see `episode.md`). Available from
the player without leaving the view.

## Default Layout

```
NavBar
[ Paged content area: Art | (Show notes placeholder) ]
[ Dot indicators ]
Position / scrubber
(Chapter Back) | Title & Chapter, From Source | (Chapter Next)   ← if chapters present
(-15s)         |   Play/Pause    | (+30s)
(Hide)         | [Player options]| (Episode options ...)
```

"Hide" collapses to mini-player and is equivalent to navigating back up the
nav stack.

## Active Playlist Persistence

The active playlist and current position are restored on cold start, so the
mini-player is visible immediately if something was playing when the app closed.
