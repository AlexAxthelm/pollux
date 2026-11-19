---
type: user-story
status: draft
feature:
  - "[[dev-environment]]"
related:
  - "[[US-0226 Reinstall device builds easily]]"
  - "[[US-0228 See when a device build expires]]"
---
As a developer
I want `make` targets to build and run on a physical device, and to build the Rust core in
release mode
so that I can verify audio, lock-screen and performance behavior on hardware, with numbers
that represent what ships

Notes:
* The Makefile has no device target and no release option: the simulator builds use a
  debug Rust core (`cargo swift package` with no `--release`), which overstated the
  player's render cost by more than 30 times. The device run used raw `xcodebuild` with
  `-xcconfig` and `cargo bin cargo-swift package --release`, with signing (a free Personal
  Team, which expires after 7 days) in a git-excluded `iOS/Local.xcconfig`. Signing settings
  must not be committed.
* `iOS/Local.xcconfig` is excluded through `.git/info/exclude` in the main checkout, not `.gitignore`.
  Worktrees share that file, but a fresh clone does not, so the device target should add the
  entry to `.gitignore` to keep signing settings out of commits. The file itself no longer
  exists in the main checkout, so the signing setup has to be recreated.
