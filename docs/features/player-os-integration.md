---
type: feature
status: accepted
priority: MVP
area: player
parent: "[[player]]"
depends:
  - "[[player]]"
---
# OS integration

Playback should behave like part of the phone: it continues in the background and
when the screen is locked, and it can be controlled from the lock screen, headphone
buttons, car and speaker integration, and Control Center.

## Behavior

- Use system media intents for playback control (lock screen, headphone buttons,
  car/speaker integration).
- The audio session is `.playback` / `spokenAudio`, with the background audio mode
  enabled, so playback continues in the background and when locked.
- Lock-screen commands: play, pause, toggle, skip ±(30/15), and scrubbing.
  Next/previous track are disabled; the skip buttons stand in for them, per podcast
  convention.
- Pause on headphone unplug. Pause on audio-session interruption (a call, Siri);
  auto-resume only if that interruption is what paused playback and the system
  says it may. A pause from unplugged headphones is never resumed automatically.
- The audio session is claimed only when audio is about to play and released when
  playback stops, so other apps are told they may resume.
- The user can select the output device (Bluetooth, AirPlay, speaker, and so on)
  through the OS route picker, available from the player options menu.

When a stream stalls, the lock-screen position should stop with it; see
[[player-streaming]].

## Decisions

- [[ADR-0008 The audio session is claimed only when audio is about to play]]
- [[ADR-0009 Next and previous track are disabled on the lock screen]]
- [[ADR-0002 Played is decided when the listener leaves an episode]] (the
  interruption and "not leaving" rules)

## Known limitations and follow-ups

### Lock-screen controls (an earlier report of invisible controls did not reproduce)

An earlier build showed the lock-screen and Control Center controls invisibly on the
author's iPhone, although they worked. On an iPhone 14 Pro Max (iOS 26.6.2) the title,
artist, artwork, 15 and 30 second skip buttons, scrubber and times all render and work,
and the scrubber does not jitter, so neither candidate fix (setting
`MPNowPlayingInfoPropertyMediaType` to audio, and setting the skip `preferredIntervals`
before the first Now Playing update) was applied. If it comes back, those are where to
start. The card is rewritten only when out of date (`NowPlayingPolicy`), so rewriting too
often is not the cause. A restored, paused episode is not on the lock screen until it has
been played once (the audio engine and remote commands are created lazily, so a
background-refresh launch stays metadata-only).

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
