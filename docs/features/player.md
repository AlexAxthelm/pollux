# Audio Playback
---
priority: MVP
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
  behavior

## Paged Content Area

The top portion of the player is a paged view (swipeable carousel with dot
indicators). MVP pages:

1. **Artwork**: episode image → feed image → placeholder. Chapter art shown
   if available and episode is in a chapter.
2. **Show notes**: see player-show-notes.md

*(Future)* Additional pages: chapter list, visualizer (see `visualizer.md`)

Note: investigate whether chapter lists are commonly embedded in show notes
as timestamps — if so, may want to detect and handle that case before
splitting into a separate page.

Note: for large screens, MVP will reamin paged, but we may want to show multiple
pages if we havew space

## Scrubber / Position

A draggable progress bar showing current position in the episode.

*(Future)* Chapter markers overlaid on the scrubber as an enhancement.

scrubber shows current time on left, and total time on right (tapping on that
switches to "remaining time)

## Controls

### Skip Buttons

Forward and back skip buttons, configurable independently:

- **Default**: 30s forward, 15s back
- **Configurable**: global default, overridable per feed (cascades global → feed)
- Button label should display the current increment (e.g. "+30" / "-15")
- Skipping past the end or beginning of the current file does not overshoot
  into the next/previous episode

### Chapter Navigation

see chapters.md

### Play / Pause

Standard toggle. Shows pause symbol when playing, play symbol when paused.

## Source Context

The player should display the active playlist/source context (e.g. "From: Music
- Boppy"). This is tappable and navigates back to the source playlist or
subscription view.

This also appears in the mini-player (see `mini-player.md`).

## Player Options Menu

Accessible from the player UI. MVP contents:

- *(placeholder for speed control — see `speed-control.md`)*
- *(placeholder for sleep timer — see `sleep-timer.md`)*
- *(placeholder for equalizer — see `equalizer.md`)*
- OS output/routing options (where available)

## Episode Options Menu

Standard three-dots episode context menu (see `episode.md`). Available from
the player without leaving the view.

## Default Layout

```
NavBar
[ Paged content area: Art | Notes ]
[ Dot indicators ]
Position / scrubber
(Chapter Back) | Title & Chapter | (Chapter Next)
(-15s)         |   Play/Pause    | (+30s)
(Hide)         | [Player options]| (Episode options ...)
```

"Hide" collapses to mini-player and is equivalent to navigating back up the
nav stack.
