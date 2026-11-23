---
type: bug
status: draft
feature:
  - "[[player-os-integration]]"
related:
  - "[[player]]"
---
# Lock-screen transport buttons are missing in the simulator

On the iOS simulator's lock screen, Pollux's Now Playing card shows the title, show,
artwork and a moving scrubber, but no play/pause or skip buttons. The row where they belong
is blank. It works on a device (the author confirmed the lock-screen controls, with the
custom skip intervals from [[settings]], on an iPhone). So this mostly costs us the ability
to check the lock screen in the simulator, and it is not known to affect listeners.
![[Pasted image 20261122165233.png]]
## Steps to reproduce

1. Play an episode of any subscription.
2. Press the lock button (AXe: `axe button lock --udid <sim>`; press it a second time to
   wake the screen).
3. Look at the Now Playing card on the lock screen.

## Actual behaviour

The card shows the title (scrolling), "Planet Money", the elapsed and remaining time and a
scrubber that advances while playing, and the volume icon. The transport row is empty: no
play/pause and no skip buttons. Waiting several seconds and locking a second time did not
change that.

## Expected behaviour

The card shows play/pause and the skip buttons, with the skip intervals from the settings
([[player-os-integration]]). See ADR-0009 for why the skip buttons stand in for next and
previous.

## Evidence

- Screenshots of the simulator lock screen after two lock cycles, as above: the card is
  populated and its clock advances (9:04 then 9:19, then 10:09), so the app is publishing
  Now Playing metadata, and only the buttons are absent.
- The in-app player on the same run showed the skip buttons as -10 and +60.
- `NowPlayingSkipIntervalTests` check that the app sets the remote commands' preferred
  intervals from the settings (first card, a change while playing, a change while paused).
  They pass, so the app does give the system the right intervals.
- An unanswered Apple forum thread reports the same kind of thing on a simulator:
  [Possible iOS 27 Simulator bug with NowPlaying: metadata only appears after second lock
  cycle, playback buttons missing](https://developer.apple.com/forums/thread/848425). It is
  a different iOS version and I found no Apple documentation saying the simulator can't draw
  the buttons, so this is a similar report, not a confirmation.
- An attempt to compare against a plain MP3 in Safari did not work: Pollux stayed the Now
  Playing app, so there is no comparison yet.

## Suspected cause

Not verified. Two readings:

1. A simulator limitation or bug, as in the forum thread. It would fit the device working.
2. The same family as the earlier report of invisible controls on a device
   ([[player-os-integration]], "Lock-screen controls"). That note lists two candidate fixes
   that were never applied: setting `MPNowPlayingInfoPropertyMediaType` to audio, and
   setting the skip `preferredIntervals` **before** the first Now Playing update.
   `NowPlayingController.publish` currently sets the card first and the intervals after,
   which is the reverse of the second one.

Cheap to try, in the simulator: set the media type, and move `updateSkipIntervals` ahead of
`center.nowPlayingInfo = info`. If the buttons appear, reading 2 was right and the order is
worth keeping on devices too.

## Environment

- Build: `feat/settings` (Debug), built with `mise run ios-xcodebuild`
- Simulator: iPhone 17 Pro, iOS 26.5
