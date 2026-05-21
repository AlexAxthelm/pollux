# Hacking on Pollux

This document covers local setup, build commands, and development workflows.
For a conceptual overview of the architecture, see `ARCHITECTURE.md`.

---

## Prerequisites

### Required

- **Rust** (stable) — managed via `rust-toolchain.toml`; install via
  [rustup](https://rustup.rs/)
- **Xcode 16+** — required for Swift 6.2 and iOS 18 deployment target
- **xcodegen** — regenerates the Xcode project from `iOS/project.yml`
  ```
  brew install xcodegen
  ```
- **cargo-swift** — exactly version 0.9.0 (the `make package` target checks
  this and will fail fast if the version is wrong)
  ```
  cargo install cargo-swift --version 0.9.0
  ```

### For Swift checks (also required in CI)

```
brew install swiftlint swiftformat
```

### Verify your setup

```
rustup show          # should show stable toolchain and Apple targets
cargo swift --version | grep 0.9.0
xcodegen --version
swiftlint --version
swiftformat --version
```

---

## Repository structure

See `ARCHITECTURE.md` for a full layout. The short version:

- `shared/` — Rust core, the only place application logic lives
- `iOS/Pollux/` — SwiftUI shell, hand-written
- `iOS/generated/` — **git-ignored**, produced by code generation; never edit
- `iOS/Pollux.xcodeproj` — **git-ignored**, produced by xcodegen; never commit

---

## Building

### Rust only

```bash
make rust-build     # cargo build
make rust-check     # cargo check (faster than build)
```

### Full iOS build (Rust → generated types → Xcode project)

```bash
make ios-build
```

This runs three steps in order:

1. `make typegen` — runs the `codegen` binary to generate Swift type stubs
   into `iOS/generated/App/`
2. `make package` — runs `cargo swift package` to compile the Rust static lib
   and wrap it as a Swift package in `iOS/generated/Shared/`
3. `make generate-project` — runs `xcodegen` to regenerate `iOS/Pollux.xcodeproj`

Run `make ios-build` whenever you change anything in `shared/src/app.rs` (new
events, view model fields, effects). You do not need to run it for changes
that only touch Swift files or only touch Rust logic that doesn't change the
public types.

### Open in Xcode

```bash
make ios-dev        # runs ios-build then opens the project in Xcode
```

Or manually: `xed iOS/` after `make ios-build`.

### Run in simulator

```bash
make ios-sim
```

This builds, installs, and launches in the first available "iPhone 14 Pro Max"
simulator. To target a different device, override `SIM_DEVICE_NAME`:

```bash
make ios-sim SIM_DEVICE_NAME="iPhone 16"
```

---

## Testing

```bash
make test           # cargo test (Rust only)
make rust-test      # same
```

Swift tests are not yet set up. Rust tests live alongside the source in
`shared/src/app.rs` using standard `#[cfg(test)]` modules. Crux provides
test helpers (`cmd.expect_one_effect().expect_render()`) that let you test
the update function without a shell.

---

## Checks and linting

Run the full check suite before opening a PR:

```bash
make check          # rust-all-checks + swift-all-checks
```

Individual targets:

```bash
make rust-check         # cargo check
make rust-lint          # cargo clippy -- -D warnings
make rust-format-check  # cargo fmt -- --check
make rust-lock-check    # cargo check --locked (ensures Cargo.lock is up to date)

make swift-lint         # swiftlint lint --strict iOS/Pollux
make swift-format-check # swiftformat --lint iOS/Pollux
```

Auto-fix formatting:

```bash
make rust-format        # cargo fmt
make swift-format       # swiftformat iOS/Pollux
```

---

## Code rules (enforced by CI)

The admin CI workflow scans for forbidden patterns. These will fail CI:

- `#[allow(...)]` or `#![allow(...)]` — fix the warning instead
- `unsafe { }`, `unsafe fn`, `unsafe impl`, `unsafe trait` — no unsafe code
- `.unwrap()` — use `?`, `expect("meaningful message")`, or explicit handling
- `.expect("")` — expect messages must be non-empty
- `try!`, `as!`, or force-unwrap operator in Swift (`!` at end of line)
- `swiftlint:disable` — fix the lint instead

---

## Workflow: adding a new feature to the core

1. Edit `shared/src/app.rs`:
   - Add variants to `Event` for any new user actions
   - Add fields to `Model` for new state
   - Add fields to `ViewModel` for anything the UI needs to display
   - Add arms to the `update()` match for the new events
   - Add new `Effect` variants if the feature requires I/O (network, storage, …)
   - Write unit tests in the `#[cfg(test)]` block

2. Run `make ios-build` to regenerate Swift types and rebuild the Xcode project.

3. Update Swift views in `iOS/Pollux/`:
   - Consume new `ViewModel` fields in `ContentView.swift` (or new view files)
   - Emit new `Event` variants via `core.update(.newEvent)`

4. Run `make check` and confirm everything passes.

---

## Workflow: regenerating types only

If you changed the public surface of the Rust core but haven't changed Swift
views yet, you can regenerate without a full rebuild:

```bash
make regenerate     # clears iOS/generated/ and re-runs typegen
make ios-build      # then package + generate-project
```

To wipe all generated artifacts and start clean:

```bash
make ios-clean      # removes iOS/generated/ and iOS/Pollux.xcodeproj
make ios-rebuild    # clean + full ios-build
```

---

## Dependency notes

- `crux_core` — the Crux framework; drives the App/Model/ViewModel/Effect
  pattern and the FFI bridge
- `uniffi` — generates the C FFI scaffolding consumed by Swift; pinned to
  exactly `0.29.4` (the version assertion in `lib.rs` will catch mismatches)
- `facet` — pinned to `=0.31` (exact version required); used for type
  introspection during code generation
- `cargo-swift` — must be exactly `0.9.0`; the `make package` target
  verifies this before running

---

## Troubleshooting

**`make package` fails with version error**
cargo-swift must be exactly 0.9.0. Run `cargo install cargo-swift --version 0.9.0 --force`.

**Xcode can't find the `Shared` or `App` packages**
Run `make ios-build` to regenerate `iOS/generated/`. Never try to add these
packages manually in Xcode.

**`xcrun simctl` can't find the simulator**
Run `xcrun simctl list devices available` to see what's available, then pass the
name via `SIM_DEVICE_NAME`.

**Type mismatch errors in Swift after changing `app.rs`**
You need to regenerate: `make ios-build`. The generated Swift types in
`iOS/generated/App/` are stale.

**`uniffi` version assertion fires**
The `uniffi` crate in `shared/Cargo.toml` and the `cargo-swift` version must
stay in sync. Currently both are pinned to 0.29.4.
