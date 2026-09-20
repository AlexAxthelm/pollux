# Pollux Architecture

This document describes the high-level architecture of Pollux for contributors
and AI assistants starting a new session. Read this before diving into the code.

---

## Overview

Pollux uses the [Crux](https://redbadger.github.io/crux/) framework: a
**shared Rust core** containing all application logic, with a thin native
shell on each platform responsible only for rendering and I/O. Currently the
only shell is iOS (Swift / SwiftUI), but the architecture is designed to
support additional platforms without changes to the core.

```
┌──────────────────────────────────────┐
│           iOS Shell (Swift)          │
│  SwiftUI views + Core ObservableObject│
└──────────┬───────────────────────────┘
           │  bincode-serialized Events / Effects
           ▼
┌──────────────────────────────────────┐
│        shared/  (Rust, Crux)         │
│  App trait · Model · ViewModel       │
│  Effect enum · Command pipeline      │
└──────────────────────────────────────┘
```

The boundary between shell and core is explicit and typed. Events flow in;
serialized Effects flow out. The core never touches UI or I/O directly.

---

## Repository Layout

```
pollux/
├── shared/                 # Rust core (Crux app)
│   ├── src/
│   │   ├── app.rs          # App trait impl: Event, update(), view()
│   │   ├── model.rs        # Model — private core state
│   │   ├── view_model.rs   # ViewModel — serializable projection for the shell
│   │   ├── effect.rs       # Effect enum (Render / Storage / Http / Download)
│   │   ├── capabilities/   # Storage, Http, Download operation & result types
│   │   ├── domain/         # Episode, Subscription
│   │   ├── feed_parser.rs  # RSS → Subscription + Episodes
│   │   ├── theme.rs        # base16 themes → ThemeView
│   │   ├── ffi.rs          # CoreFFI — the C/UniFFI/WASM bridge struct
│   │   ├── lib.rs          # Crate root; re-exports; UniFFI scaffolding
│   │   └── bin/
│   │       └── codegen.rs  # CLI binary: generates Swift/Kotlin/TS types
│   └── Cargo.toml
├── iOS/
│   ├── Pollux/             # SwiftUI app
│   │   ├── Pollux.swift          # @main entry point
│   │   ├── ContentView.swift     # library + subscribe flow
│   │   ├── core.swift            # Core ObservableObject: drives CoreFFI
│   │   ├── DatabaseManager.swift # Storage effect handler (GRDB/SQLite)
│   │   ├── DownloadManager.swift # Download effect handler (URLSession)
│   │   └── Theme.swift           # ThemeView → SwiftUI colors
│   ├── generated/          # ← git-ignored; produced by `make typegen` + `make package`
│   │   ├── App/            # Swift type stubs (Event, ViewModel, Request, …)
│   │   └── Shared/         # Swift package wrapping the compiled Rust static lib
│   └── project.yml         # XcodeGen spec (source of truth for the Xcode project)
├── docs/
│   ├── ARCHITECTURE.md     # this file
│   ├── HACKING.md          # developer setup and workflow
│   ├── ROADMAP.md          # phased implementation plan
│   └── features/           # per-feature specs (priority + depends tags)
├── Cargo.toml              # workspace root
├── Cargo.lock
├── Makefile                # primary task runner (see HACKING.md)
├── rust-toolchain.toml     # pins stable Rust + Apple targets
└── .swift-version          # pins Swift 6.2
```

`iOS/*.xcodeproj` and `iOS/generated/` are both **git-ignored**. They are
regenerated during the build; never commit them.

---

## The Crux Pattern

Crux implements a strict unidirectional data flow:

```
Event → update() → Command<Effect, Event>
                        │
                   Shell processes effects
                        │
              resolve() → more Commands (if async)
                        │
                    view() → ViewModel → UI render
```

### Core types (in `shared/src/app.rs`)

| Type | Role |
|---|---|
| `Pollux` | Zero-sized struct implementing `crux_core::App` |
| `Event` | All inputs the shell feeds in — user actions (subscribe/refresh a feed, select a subscription, sort/download/cancel an episode, set the theme) and the async results of storage/HTTP/download effects |
| `Model` | Private application state (never leaves the core); see `model.rs` |
| `ViewModel` | Public, serializable snapshot of state for the shell to render; see `view_model.rs` |
| `Effect` | Side-effects the shell must handle: `Render`, `Storage` (SQLite via GRDB), `Http` (feed fetch), `Download` (episode audio); see `effect.rs` |

`Model` and `ViewModel` are deliberately separate. `Model` is private and
mutable; `ViewModel` is a derived, serializable read-only projection.

### FFI bridge (`shared/src/ffi.rs`)

`CoreFFI` is the single struct exposed to the shell. It wraps
`crux_core::Bridge<Pollux>` and exposes three methods:

- `update(data: &[u8]) -> Vec<u8>` — send a bincode-serialized `Event`, get
  back bincode-serialized `[Request]`
- `resolve(id: u32, data: &[u8]) -> Vec<u8>` — deliver the result of an async
  effect
- `view() -> Vec<u8>` — get the current bincode-serialized `ViewModel`

The shell is responsible for deserializing effects and dispatching them. The
synchronous `Render` effect is handled inline; the async `Storage`, `Http`, and
`Download` effects are dispatched to platform APIs and their results delivered
back via `resolve()`.

### iOS shell (`iOS/Pollux/core.swift`)

`Core` is a `@MainActor ObservableObject`. On `update(_:)`:

1. Serializes the `Event` with bincode
2. Calls `CoreFFI.update(data:)`
3. Deserializes the returned `[Request]`
4. Dispatches each `Request` via `processEffect(_:)`: `.render` deserializes and
   publishes the `ViewModel`; `.storage`, `.http`, and `.download` run their
   async work and feed the result back through `resolve()`. Storage runs on a
   single serial consumer so writes apply in submission order.

SwiftUI views observe `core.view` and re-render automatically.

---

## Code Generation

The Swift types (`Event`, `ViewModel`, `Request`, and the `App` Swift package
wrapper) are **generated** — never written by hand. The pipeline:

```
make typegen   →  cargo run --bin codegen (--features codegen,facet_typegen)
                  writes Swift type stubs to iOS/generated/App/

make package   →  cargo swift package (requires cargo-swift 0.9.0)
                  compiles the Rust static library and wraps it as a Swift package
                  writes to iOS/generated/Shared/

make generate-project  →  xcodegen --spec iOS/project.yml
                           regenerates iOS/Pollux.xcodeproj
```

`make ios-build` runs all three in order. Run it whenever the core's public
types change — new `Event`/`Effect` variants, `ViewModel` fields, or capability
operation/result types in `shared/src/`.

Type serialization uses **bincode** (compact binary) at the FFI boundary and
**serde** for all types. `facet` is used to drive type generation.

---

## Toolchain

| Tool | Version / notes |
|---|---|
| Rust | stable (pinned via `rust-toolchain.toml`) |
| Swift | 6.2 (pinned via `.swift-version`) |
| cargo-swift | exactly 0.9.0 (checked in `make package`) |
| xcodegen | any recent version |
| swiftlint | installed via Homebrew for CI |
| swiftformat | installed via Homebrew for CI |

Rust targets compiled: `aarch64-apple-darwin`, `aarch64-apple-ios`,
`aarch64-apple-ios-sim`, `x86_64-apple-ios`, `x86_64-apple-darwin`.

---

## CI

GitHub Actions runs on push and PR to `main`:

- **Rust checks**: `cargo check`, `cargo test`, `cargo clippy -D warnings`,
  `cargo fmt --check`, `cargo check --locked`
- **Swift checks**: `swiftlint lint --strict`, `swiftformat --lint`
- **Admin**: forbidden-pattern scan (no lint suppressions, no `unsafe`, no
  force-unwrap calls, no force-try/cast in Swift, etc.)

All CI targets in `.github/workflows/` are reusable `workflow_call` workflows
composed by `rust.yml`, `swift.yml`, and `admin.yml`.

---

## Current State

The MVP foundation is in place: RSS feed parsing and subscribing, a library and
per-feed episode list, and an offline **download manager** (serial queue, live
progress, resume-on-launch, cancel). A base16 **theming** system is wired
through the `ViewModel`. State lives in the Rust core; the iOS shell handles
SQLite (GRDB), HTTP, downloads (URLSession), and rendering.

Still to come (see `docs/ROADMAP.md`): audio playback, playlists (the core
differentiator), a Settings screen, and feed refresh. Sync is out of scope for
MVP and v1.0. Feature intent and priorities live in `docs/features/`.

When adding a new domain feature, the typical change surface is:

1. Extend the core types in `shared/src/` — `Event`/`update()`/`view()` in
   `app.rs`, plus `model.rs`, `view_model.rs`, `effect.rs`, and `capabilities/`
   as needed
2. Run `make ios-build` to regenerate Swift types and rebuild the Xcode project
3. Update SwiftUI views in `iOS/Pollux/` to consume the new `ViewModel` fields
   and emit the new `Event` variants
