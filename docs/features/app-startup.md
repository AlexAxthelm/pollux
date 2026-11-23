---
type: feature
status: draft
priority: mid
area: app
depends:
  - "[[library]]"
  - "[[mini-player]]"
  - "[[player-progress]]"
---
# App startup

How fast the app opens, what it shows first, and that it never starts sound by itself. This
file is drawn from the stories below; it has not been specified further.

## What is wanted

- A cold launch to a usable screen in **under a second** (a target, "would be nice", not a
  hard limit) ([[US-0237 The app opens in under a second]]).
- **No loading screen.** A cold launch drops straight into the library, and coming back to
  an app that is already open shows what was on screen
  ([[US-0238 The app opens straight into the library without a loading screen]]).
- **Play at once** from the mini-player: the episode it shows is cued up and ready by the time
  the play button appears ([[US-0239 Play the mini-player's episode the moment I tap play]]).
- **Sound only when asked.** Entering the app never starts playback. Returning to an app that
  is already playing carries on ([[US-0240 Sound starts only when I ask for it]]).
- **A steady layout.** What is visible is shown in one go, with the UI blocked until it has
  loaded, so nothing jumps as data arrives. What isn't visible may load in the background,
  quickly ([[US-0241 The screen does not shift as data loads]]).

## Current state

Nothing here has been measured: no launch timing is recorded in these docs, and none was taken
for this note. What the code does, read from it and not timed:

- At launch the core asks for the subscriptions, the saved play context and the saved settings
  together. The library screen shows a **spinner while the subscriptions load**
  (`ContentView`), so there is a brief loading state today.
- The mini-player comes back paused at its place once the play context has loaded
  ([[player-progress]], [[ADR-0012 The play context is restored on launch with capped save retries]]).
  The restored episode is **not loaded into the audio engine**, and the engine and the
  lock-screen controls are created on the first play request, so the first Play does that work
  first. Nothing autoplays on launch: the audio session is claimed only when audio is about to
  play ([[ADR-0008 The audio session is claimed only when audio is about to play]]).
- A play or skip pressed in the first moments after launch waits for the saved settings to be
  read, so it uses the saved skip and rewind. That is milliseconds, and a failed read lets it go
  ahead with the defaults ([[settings]]).
- Navigation state is held in memory (`NavigationPath`). It is not restored after the app is
  killed, so a cold launch opens on the library, as wanted. A backgrounded app that stays in
  memory keeps what was on screen.

## Contradictions

#needs-decision The mini-player's episode should be **cued up and ready to play** by the time
the play button shows ([[US-0239 Play the mini-player's episode the moment I tap play]]), but
[[player-os-integration]] says the audio engine and remote commands are created lazily, on the
first play, so that a background-refresh launch stays metadata-only. Both are kept as written.
Reconciling them is open: [[ADR-0008 The audio session is claimed only when audio is about to play]]
already allows a paused load without claiming the audio session, so cueing the restored episode
would not interrupt other apps' audio. The costs to weigh (inferred, not measured) are creating
the engine at every launch, including a background-refresh one, and the lock-screen card
appearing without the listener having played anything.

## Open questions

- How is it measured? From the tap on the icon to the library being visible and usable, on a
  device, in a release build. The simulator and debug builds overstate costs (see
  [[player-performance]]).
- What counts as "visible" for the library: every subscription, or only the first screenful? A
  large library may not load in one go.
- Artwork loads after the rows. Reserving the artwork's space keeps the layout steady, but the
  picture still arrives later. Is that acceptable under the steady-layout story?
- Cueing the restored episode (see Contradictions) may mean creating the engine earlier, or
  loading it in the background after the first screen. Which, and whether a background-refresh
  launch is exempt.
- Whether to record a decision as an ADR once the approach is chosen. The stories say what is
  wanted, not how, so there is no ADR yet.

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
