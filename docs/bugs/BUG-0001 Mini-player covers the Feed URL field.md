---
type: bug
status: draft
feature:
  - "[[mini-player]]"
  - "[[library]]"
related: []
---
# Mini-player covers the Feed URL field

When an episode is loaded in the player, the [[mini-player]] bar is drawn over
the "Feed URL" text field and the "Add" button at the bottom of the
[[library]] screen. Taps on them go to the mini-player instead, so a listener
with an episode loaded can't add a podcast.

## Steps to reproduce

1. Play any episode, so the mini-player is showing. Starting playback in the
   same session is enough, and so is relaunching the app with a previously
   played episode restored.
2. Go to the Library screen.
3. Tap the "Feed URL" field.

## Actual behaviour

The tap opens the full-screen player and the text field never gets focus. The
"Add" button sits under the mini-player's Play button, so a tap where "Add" is
would hit Play instead (only the text field was tried).

Tapping by accessibility element reference behaves the same way. An automation
tool reported "text typed successfully" while the full player opened and the
field stayed empty, because the tap lands on whatever is on top at that position.

## Expected behaviour

The URL row sits above the mini-player and stays tappable. The
[[mini-player]] spec anchors the bar to the bottom of the screen, and Library
content should end above it.

## Evidence

Accessibility tree on an iPhone 14 Pro Max simulator (iOS 26.5, 430×932 pt):

| Element | Frame (x, y, w, h) |
|---|---|
| TextField "Feed URL" | 16, 848, 359, 34 |
| Button "Add" | 383, 855, 31, 20 |
| Button "Now playing: …" (mini-player) | 16, 844, 342, 44 |
| Button "Play" (mini-player) | 385, 857, 16, 18 |

The mini-player's frames fall inside the URL row's. Tapping the small strip of
the field past the "Now playing" button (x≈366) still opened the player, so the
bar seems to cover that part too.

With nothing loaded, as on a fresh install, the field is at the same position
and works normally. Only the mini-player's presence triggers the bug.

## Suspected cause

Not verified. `Pollux.swift` attaches the mini-player with
`.safeAreaInset(edge: .bottom)` on `ContentView`, which wraps a
`NavigationStack`. The inset doesn't seem to reach the `VStack` inside the
stack, since the URL row ends up at y=848 and not above the bar at y=844. Moving
the inset inside the `NavigationStack`, or onto the `VStack`, may fix it, but
that hasn't been tried.

## Environment

- Found on `main` at `e8cd0b7` (merge of PR #23), built with `make ios-xcodebuild`
- iPhone 14 Pro Max simulator, iOS 26.5. Not tried on a device.
