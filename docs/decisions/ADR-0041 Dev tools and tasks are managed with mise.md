---
type: decision
status: draft
feature:
  - "[[dev-environment]]"
related:
  - "[[US-0187 Check for build tools]]"
  - "[[US-0191 Make targets for physical devices]]"
---
# ADR-0041 Dev tools and tasks are managed with mise

## Context

Setup needed several install paths: `brew install` for xcodegen, xcbeautify, swiftlint
and swiftformat, `cargo install cargo-run-bin`, and rustup. CI installed swiftlint and
swiftformat from brew, so local and CI versions could differ. Task commands lived in a
Makefile, but almost all its targets were phony. It used none of make's file-timestamp
graph, and it carried make-specific workarounds: `SHELL := /bin/bash` for macOS's GNU Make
3.81, `set -o pipefail;` in every xcodebuild recipe, `$$` escaping, and a `SIM_ID` assigned
with `$(shell …)`, which ran `xcrun simctl` on every invocation, even `make rust-test`.

## Decision

mise manages the developer tools and runs the tasks.

* `mise.toml` pins xcodegen, xcbeautify, swiftlint and swiftformat, limited to macOS. They
  are pinned to the versions the Swift checks were last run with.
* Tasks keep the old Makefile names (`mise run rust-test`, `mise run ios-sim`, and so on).
  Short ones are in `mise.toml`; longer recipes are scripts in `mise-tasks/`, sharing
  settings through `scripts/ios-env.sh`.
* Settings you may override are environment variables (`SIM_DEVICE_NAME`, `DEVICE_ID`).
* Ordered chains use `run` lists. `depends` is used only where a task just needs a
  prerequisite to have run.
* CI runs the same tasks through `jdx/mise-action`. The Rust jobs install only the task
  runner. The Swift jobs install only the tool they use.
* The Makefile is removed.

Rust stays with rustup and `rust-toolchain.toml`. cargo-swift stays with cargo-run-bin and
the pin in the root `Cargo.toml`.

## Consequences

* mise is now a prerequisite, for contributors and for CI. Make needed no install.
* Tool versions are pinned in one file and shared by local and CI runs. A bump is a
  deliberate edit to `mise.toml`.
* The simulator lookup runs only in tasks that need a simulator, and a missing simulator
  fails early with a hint.
* Xcode, the Swift toolchain and the simulator runtime are still manual.
* The CI change cannot be tested locally. The first PR run is its test.
* Open: running independent checks in parallel (`depends`), moving cargo-swift from
  cargo-run-bin to mise, and whether mise's auto-install makes a preflight check
  ([[US-0187 Check for build tools]]) unnecessary.
* Revisit if mise's task features change or the file tasks grow past what a script
  directory suits.
