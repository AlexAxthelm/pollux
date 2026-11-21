---
type: user-story
status: draft
feature:
  - "[[dev-environment]]"
related:
  - "[[US-0230 Reinstall device builds easily]]"
  - "[[US-0232 See when a device build expires]]"
---
As a developer
I want build tasks to build and run on a physical device, and to build the Rust core in
release mode
so that I can verify audio, lock-screen and performance behavior on hardware, with numbers
that represent what ships

Notes:
* Built: `mise run ios-device` builds, installs and launches a Debug build on a paired phone
  ([[US-0230 Reinstall device builds easily]]). Signing, and the bundle id to replace, come
  from the git-ignored `iOS/Local.xcconfig`, which is now in `.gitignore`. Setup is in
  [[HACKING]].
* Not built: the release option. The simulator and device builds both use a debug Rust core
  (`cargo swift package` with no `--release`), which overstated the player's render cost by
  more than 30 times. An earlier device run got representative numbers with
  `cargo bin cargo-swift package --release`. Until a release option exists, don't read
  performance numbers from `mise run ios-device`.
* Signing settings must not be committed. A free Personal Team's profile expires after
  7 days ([[US-0232 See when a device build expires]]).
