---
type: feature
status: draft
priority: MVP
area: player
depends:
  - "[[episode]]"
  - "[[mini-player]]"
  - "[[subscription]]"
  - "[[settings]]"
---
# Audio Playback

The player is the primary listening interface. It integrates with the OS media
system where available, and surfaces controls for the currently active episode
and playlist.


## Parts

- [[player-progress]]: resume, saving position, played detection, restore on launch
- [[player-streaming]]: streaming and local playback, the swap, download edge cases
- [[player-os-integration]]: background playback, lock screen, interruptions, output routing
- [[player-performance]]: render cost and resource use
- [[mini-player]]: the compact control shown when the full player is closed
- [[player-show-notes]]: the show notes page

## OS Integration

Lock screen, headphone buttons, car and speaker integration, background playback
and audio-session handling are in [[player-os-integration]]. Resuming where the
listener left off is in [[player-progress]].

## Paged Content Area

The top portion of the player is a paged view (swipeable carousel with dot
indicators). MVP pages:

1. **Artwork**: episode image → feed image → placeholder. Chapter art shown
   if available and episode is in a chapter (see `episode-chapters.md`).
2. **Show notes**: the episode's show notes, rendered as rich text (the same
   renderer as the episode detail page) and scrollable. The page, and the dot
   indicators, are left out when the episode has no show notes. Timestamp
   detection and tappable timestamps are not built yet; see
   `player-show-notes.md`.

*(Future)* Additional pages: chapter list, visualizer (see `visualizer.md`)

The artwork fills the width of the player (as large a square as fits). A
user-configurable player layout is a possible later feature; the default layout
below is the only one for now.

Primary controls (play/pause, skip) are drawn in the theme's accent color; the
secondary row (hide, output routing, options) uses the text color.

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
[ Paged content area: Art | Show notes (if any) ]
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

Restore details are in [[player-progress]].

## Presentation

The full player is a full-screen cover from the root; the mini-player is a bottom
inset on the root. Tapping "From: …" dismisses the cover and shows the podcast.

## Errors

A playback failure leaves playback paused with a transient banner in the player
and mini-player; pressing play retries.

## Device verification

Verified in the simulator: streaming, skip, pause and resume with rewind, restore at launch,
"From:" navigation, show notes, the interruption and route-change mapping (by posting the
system's notifications), and the engine's end, failure and tick behavior.

Verified on an iPhone 14 Pro Max (iOS 26.6.2): lock-screen rendering and controls; background
audio with a long locked playback (the position is right on unlock), and a relaunch restores
the mini-player paused, 3 s back; interruptions (Siri resumes, a call resumes if it was
playing and does not after a manual pause, unplugging headphones pauses and stays paused); the
route picker opens and works; the stream-to-local swap is audible but small and not jarring,
with position and playback continuing; the offline error banner (it stays until the next
successful action, which is accepted) with the failed-download triangle and Retry still on
the row; and the render cost (above).

Not verified: switching outputs through the route picker (Bluetooth or AirPlay), and the
Energy Log. The test host's AVPlayer completes seeks at once, so the timing of a real seek is
not covered by tests.

## Decisions

Recorded so they are not silently reversed. The core owns all player policy
(`shared/src/player.rs`); the shell's `PlaybackManager` only drives the audio
engine. Each decision has its own note in `decisions/`.

- [[ADR-0001 Engine events carry a session id]]
- [[ADR-0002 Played is decided when the listener leaves an episode]]
- [[ADR-0003 Streaming starts at once and swaps to the download]]
- [[ADR-0004 Playback never downloads against the listener's wishes]]
- [[ADR-0005 Position persistence cadence and resume rewind]]
- [[ADR-0006 No auto-advance until playlists exist]]
- [[ADR-0007 Playback source is separate from the episode's feed]]
- [[ADR-0008 The audio session is claimed only when audio is about to play]]
- [[ADR-0009 Next and previous track are disabled on the lock screen]]
- [[ADR-0010 The engine's duration beats the feed's]]
- [[ADR-0011 Playback errors pause playback and retry on play]]
- [[ADR-0012 The play context is restored on launch with capped save retries]]
- [[ADR-0013 Player state is separate from the library view]]

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
