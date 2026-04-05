# Audio Playback
---
priority: MVP
---

The app should integrate with the OS media system, where available.
* Use system media intents for 

## UI

The main player interface should present a standard UI for an audio player:

* Large "album art":
    * Episode image, fallback to feed image. If there is an embedded image for
      the capter, that should be shown.
* Play / pause button
    * Below album art
    * main UI color
    * When playing, shows the "pause" symbol, when not playeing, shows the play
      button.
* Forward / back buttons
    * To either side, in line with play/pause
    * main ui color
    * ideally icon is some arrow indicating a jump or skip, showing the
      increment of time (default 30s, but this will be configurable). if that
      isn't an option, fall back to normal skip buttons (`>>|`)
    * Skips forward/backward in current file by a set increment
        * skipping past end/beginning of current file does not adjust position
          in next one (no overshoot)
* Episode Title
* A "hide" button puts the player into mini-player. in general, this acts the
  same as the "back" (moving one up in the nav stack), but is on the bottom
  (easy nav)
* "Episode Options" is the standard "three dots" menu for the episode
* Player options is for things like sleep timer, equializer, speed, and any
  output menus available (integration with OS)

Expected default layout:
```
NavBar
Episode art / Viz / Info / Other "big" things
Position
(Chapter Back) | Title & Chapter | (Chapter Next)
(Back 30s) | Play/Pause | (Forward 30s)
(Hide) | [Player options] | (Episode options `...`)
```


