---
type: reference
---
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
│   │   ├── effect.rs       # Effect enum (Render / Storage / Http / Download / Player)
│   │   ├── capabilities/   # Storage, Http, Download, Player operation & result types
│   │   ├── domain/         # Episode, EpisodeSource, Subscription
│   │   ├── player.rs       # Player policy: resume, skip, played; the engine is the shell's
│   │   ├── player/         # Player tests, split by concern
│   │   ├── feed_parser.rs  # RSS → Subscription + Episodes
│   │   ├── html.rs         # Show-notes and description text helpers
│   │   ├── defaults.rs     # Built-in default values (for example, when an episode counts as played)
│   │   ├── theme.rs        # base16 themes → ThemeView
│   │   ├── ffi.rs          # CoreFFI — the C/UniFFI/WASM bridge struct
│   │   ├── lib.rs          # Crate root; re-exports; UniFFI scaffolding
│   │   └── bin/
│   │       └── codegen.rs  # CLI binary: generates Swift/Kotlin/TS types
│   └── Cargo.toml
├── iOS/
│   ├── Pollux/             # SwiftUI app
│   │   ├── Pollux.swift          # @main entry point; applies the theme, hosts the app
│   │   ├── ContentView.swift     # library + subscribe flow
│   │   ├── core.swift            # Core ObservableObject: drives CoreFFI
│   │   ├── PlayerState.swift     # the active player, kept apart from Core.view
│   │   ├── *Screen / *View / *Row / *Bar .swift   # SwiftUI views (library, details,
│   │   │                         #   episode, full player, mini-player)
│   │   ├── DatabaseManager*.swift, DatabaseMigrations.swift   # Storage effect handler (GRDB/SQLite)
│   │   ├── DownloadManager.swift # Download effect handler (URLSession)
│   │   ├── FeedFetcher.swift     # Http effect handler for feeds
│   │   ├── BackgroundRefresh.swift   # best-effort BGAppRefreshTask
│   │   ├── PlaybackManager*.swift    # Player effect handler: AVPlayer and the audio session
│   │   ├── NowPlaying*.swift     # lock screen, Control Center, headphone and car controls
│   │   └── Theme.swift           # ThemeView → SwiftUI colors
│   ├── PolluxTests/        # Swift tests (swift-testing)
│   ├── generated/          # ← git-ignored; produced by `make typegen` + `make package`
│   │   ├── App/            # Swift type stubs (Event, ViewModel, Request, …)
│   │   └── Shared/         # Swift package wrapping the compiled Rust static lib
│   └── project.yml         # XcodeGen spec (source of truth for the Xcode project)
├── docs/                   # Obsidian vault; see docs/README.md
│   ├── ARCHITECTURE.md     # this file
│   ├── DATA_MODEL.md       # entities and their relationships
│   ├── DESIGN_PRINCIPLES.md, GLOSSARY.md, HACKING.md, ROADMAP.md
│   ├── features/           # per-feature specs (priority + depends tags)
│   ├── user_stories/       # one note per user story
│   ├── decisions/          # one note per decision (ADR)
│   ├── bugs/               # one note per bug
│   └── policies/           # CODE_OF_CONDUCT, CONTRIBUTING
├── .github/workflows/      # CI (see below)
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
| `Event` | All inputs the shell feeds in — user actions (subscribe/refresh a feed, select a subscription, sort/download/cancel an episode, play/pause/seek/skip, set the theme), the async results of storage/HTTP/download effects, and news from the audio engine (position ticks, duration, end of file, failures, interruptions) |
| `Model` | Private application state (never leaves the core); see `model.rs` |
| `ViewModel` | Public, serializable snapshot of state for the shell to render; see `view_model.rs` |
| `Effect` | Side-effects the shell must handle: `Render`, `Storage` (SQLite via GRDB), `Http` (feed fetch), `Download` (episode audio), `Player` (commands for the audio engine); see `effect.rs` |

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
synchronous `Render` effect is handled inline; the async `Storage`, `Http`, `Download`, and
`Player` effects are dispatched to platform APIs and their results delivered
back via `resolve()`.

### iOS shell (`iOS/Pollux/core.swift`)

`Core` is a `@MainActor ObservableObject`. On `update(_:)`:

1. Serializes the `Event` with bincode
2. Calls `CoreFFI.update(data:)`
3. Deserializes the returned `[Request]`
4. Dispatches each `Request` via `processEffect(_:)`: `.render` deserializes and
   publishes the `ViewModel`; `.storage`, `.http`, `.download`, and `.player` run their
   async work and feed the result back through `resolve()`. Storage runs on a
   single serial consumer so writes apply in submission order.

SwiftUI views observe `core.view` and re-render automatically. The player's own state is
split out of the view model and held in `PlayerState`, so the once-a-second position
ticks do not wake the library views; see
[[ADR-0013 Player state is separate from the library view]].

### Who decides what

The core owns policy; the shell owns engines and I/O. The audio engine (`AVPlayer`, the
audio session, the lock screen) lives in the shell, and the core decides when to resume,
how far to skip and when an episode counts as played, telling the engine what to do through
the `Player` effect. Engine news comes back as ordinary `Event`s. See `capabilities/player.rs`
and [[ADR-0001 Engine events carry a session id]].

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
| cargo-swift | exactly 0.9.0, pinned in `[workspace.metadata.bin]` and run through cargo-run-bin (see [[HACKING]]) |
| iOS deployment target | 18.0 |
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
composed by `rust.yml`, `swift.yml`, and `admin.yml`. `admin.yml` also runs nightly
and can be started by hand.

---

## Current State

The MVP foundation is in place: RSS feed parsing and subscribing, **feed refresh**
(manual, on foreground, and best-effort in the background, using conditional GETs;
see [[subscription]] and [[subscription-refresh]]), a library and per-feed episode list with
an episode detail view and show notes, an offline **download manager** (serial queue, live
progress, resume on first foreground, cancel), and **audio playback**: the engine and player
UI, the mini-player, lock-screen and headphone controls, and restore of the active episode
on launch (the active source is a single subscription, with no auto-advance; see [[player]]). A base16
**theming** system is wired through the `ViewModel`. State lives in the Rust core; the iOS
shell handles SQLite (GRDB), HTTP, downloads (URLSession), the audio engine, and rendering.

Still to come (see [[ROADMAP]]): playlists (the core differentiator) and a Settings screen.
Sync is out of scope for MVP and v1.0. Feature intent and priorities live in
`docs/features/`.

### Decisions

Why things are built this way is recorded in `docs/decisions/`. The ones that shape the
architecture:

- [[ADR-0001 Engine events carry a session id]]
- [[ADR-0013 Player state is separate from the library view]]
- [[ADR-0024 The order the library shows is decided by the core]]
- [[ADR-0032 The core owns palette data and the active theme, and the shell resolves colors]]
- [[ADR-0036 The library is the only top-level destination, and the player is a cover over it]]
- [[ADR-0040 The database schema is a single baseline until the first release]]

When adding a new domain feature, the typical change surface is:

1. Extend the core types in `shared/src/` — `Event`/`update()`/`view()` in
   `app.rs`, plus `model.rs`, `view_model.rs`, `effect.rs`, and `capabilities/`
   as needed
2. Run `make ios-build` to regenerate Swift types and rebuild the Xcode project
3. Update SwiftUI views in `iOS/Pollux/` to consume the new `ViewModel` fields
   and emit the new `Event` variants
