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

* mise installs the Rust toolchain, reading `rust-toolchain.toml` so it stays the one place
  the channel, components and targets are written. mise installs it through rustup.
* `mise.toml` pins xcodegen, xcbeautify, swiftlint and swiftformat, limited to macOS. They
  are pinned to the versions the Swift checks were last run with.
* Tasks keep the old Makefile names (`mise run rust-test`, `mise run ios-sim`, and so on).
  Short ones are in `mise.toml`; longer recipes are scripts in `mise-tasks/`, sharing
  settings through `scripts/ios-env.sh`.
* Settings you may override are environment variables (`SIM_DEVICE_NAME`, `DEVICE_ID`).
* Ordered chains use `run` lists. `depends` is used only where a task just needs a
  prerequisite to have run.
* CI runs the same tasks through `jdx/mise-action`. The Rust jobs get their toolchain from
  it, replacing `dtolnay/rust-toolchain`. The Swift jobs install only the tool they use.
* The Makefile is removed.

cargo-swift stays with cargo-run-bin and the pin in the root `Cargo.toml`.

## Consequences

* mise is now a prerequisite, for contributors and for CI. Make needed no install.
* Tool versions are pinned in one file and shared by local and CI runs. A bump is a
  deliberate edit to `mise.toml`.
* The simulator lookup runs only in tasks that need a simulator, and a missing simulator
  fails early with a hint.
* Xcode, the Swift toolchain and the simulator runtime are still manual.
* mise sets `RUSTUP_TOOLCHAIN`, which rustup ranks above `rust-toolchain.toml`. That is
  harmless while mise reads the same file, but a `rust` entry added to `mise.toml` or a
  global config would override it, so don't add one.
* `stable` still floats, so a new stable release can break CI with new lints. Pinning a
  version is a separate decision.
* CI cannot be tested locally. The PR run is its test, and it passed for the lint, format and
  Rust jobs (PR #28).
* Open: running independent checks in parallel (`depends`), pinning the Rust version,
  moving cargo-swift from cargo-run-bin to mise, and whether mise's auto-install makes a preflight check
  ([[US-0187 Check for build tools]]) unnecessary.
* Revisit if mise's task features change or the file tasks grow past what a script
  directory suits.
