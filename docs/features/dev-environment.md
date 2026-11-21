---
type: feature
status: draft
priority: low
area: dev-environment
depends: []
---
# Dev environment

How a developer builds, checks and runs Pollux locally. Setup and commands are in
[[HACKING]]; this note holds the requirements for improving them. Tools and tasks are
managed with mise ([[ADR-0041 Dev tools and tasks are managed with mise]]).

## Not yet

* Build tool preflight check ([[US-0187 Check for build tools]]).
* A local forbidden-patterns scan ([[US-0190 Admin checks run locally]]).
* A release-mode Rust core for device builds, and a device-run target that does not need
  the one-time signing setup ([[US-0191 Make targets for physical devices]]).
* Showing when a device build expires ([[US-0232 See when a device build expires]]).
* Running independent checks in parallel, pinning the Rust version, and moving cargo-swift
  from cargo-run-bin to mise. None is done.

## As built

* mise pins xcodegen, xcbeautify, swiftlint and swiftformat (`mise.toml`) and runs every
  task that used to be a make target, under the same names (`mise run <task>`). The Makefile
  is gone. Checked locally: `mise run check`, a cold `mise run ios-xcodebuild`, and
  `mise run ios-test` all pass. Not run: `ios-device` with a phone connected, the `ios-sim`
  launch step, `ios-dev`, `ios-rebuild`, `clean` and `format`. CI now calls the same tasks
  through `jdx/mise-action`, which ran green on PR #28 for the lint, format and Rust jobs.
* mise installs Rust from `rust-toolchain.toml`, locally and in CI, replacing
  `dtolnay/rust-toolchain`. The Rust CI jobs passed on this setup (PR #28), installing the
  toolchain in about 13 seconds. Not yet checked: a first install on a machine without
  Rust. `clippy` was added to the toolchain file's components,
  because it was missing for anyone without a default-profile install.
* `mise run ios-device` builds, installs over the existing app and launches it on the first
  paired iPhone ([[US-0230 Reinstall device builds easily]],
  [[US-0231 Reinstall keeps app data]]). Verified on a physical iPhone 14 Pro Max: the
  reinstall kept subscriptions once `PRODUCT_BUNDLE_IDENTIFIER` in `iOS/Local.xcconfig`
  matched the installed app. Setup is in [[HACKING]].

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
