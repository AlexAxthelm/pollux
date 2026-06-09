# Phase 0, Part 1 — Domain Model + SQL Storage Foundation

## Context

The Pollux Rust core is currently counter-app scaffolding (`Increment/Decrement/Reset`).
No domain types exist. This plan replaces that scaffolding with the real data model
and establishes the SQL storage Effect boundary end-to-end — Rust typed operations
through to iOS SQLite via GRDB.

**GRDB** is a Swift library that wraps SQLite (a file-based database built into iOS)
with a type-safe query API, migrations, and async/await support.

This is the prerequisite for everything else in Phase 0. RSS parsing, the download
manager, and the full UI all depend on these foundations being in place.

### Architecture note: Effects and Events

The Rust core is pure logic — no I/O. It communicates through two typed channels:
- **Events** come *into* the core: "app opened", "DB returned these subscriptions"
- **Effects** go *out* from the core: "query the DB for subscriptions", "re-render the UI"

The iOS shell handles all I/O, sends results back as Events. This makes the entire
app logic unit-testable in pure Rust with no simulator.

`Model` is the core's private in-memory state (never leaves Rust). `ViewModel` is a
derived, serializable snapshot of what iOS needs to render. They are deliberately separate.

---

## Scope

**In scope:**
- Domain types: `Subscription`, `Episode`, `PlaybackStatus`, `DownloadStatus`
- Settings: **constants only** (no settings struct; each gets a `// TODO: settings cascade` note)
- `StorageOperation` typed enum + `StorageResult` enum (the Rust/iOS boundary)
- `Effect` expanded with `Storage(StorageOperation)` variant
- `Event` variants for the library loading flow: `Init`, `SubscriptionsLoaded(StorageResult)`
- `Model` holding in-memory subscriptions list + loading/error state
- `ViewModel` for library view: list of `SubscriptionSummary`
- iOS: GRDB added as Swift package dependency
- iOS: `DatabaseManager.swift` — schema, migrations, `execute(_:)` dispatcher
- iOS: `processEffect` expanded for `.storage` case (async, calls DatabaseManager, resolves)
- iOS: `ContentView.swift` replaced with a stub library view (subscription list or empty state)
- All existing Rust tests updated; new tests for the Init and SubscriptionsLoaded events

**Not in scope (next PR):**
- Network Effect / RSS parsing
- Subscribe-from-URL flow
- Download manager
- Navigation structure
- Feed refresh

**ID strategy:**
- `Subscription`: UUID (as `String`). Feed URLs can change (server migrations, redirects);
  UUID is the stable internal key. `feed_url` stored as a separate unique column.
  Add `uuid` crate to `shared/Cargo.toml` so IDs are generated in the Rust core.
- `Episode`: surrogate UUID primary key (`id`), with `feed_guid` storing the RSS `<guid>` string
  and a `UNIQUE (subscription_id, feed_guid)` constraint enforcing per-feed uniqueness.
  RSS guids are only guaranteed unique within a single feed, not globally — using guid alone
  as a primary key risks collisions across subscriptions.

---

## File Changes

### Rust (`shared/`)

Split `app.rs` into modules as it grows. New structure:

```
shared/src/
├── app.rs              # App trait impl — thin, delegates to handlers
├── ffi.rs              # unchanged
├── lib.rs              # re-exports (add domain/* and capabilities/*)
├── capabilities/
│   └── storage.rs      # StorageOperation, StorageResult, Operation impl
├── domain/
│   ├── mod.rs          # pub use *
│   ├── episode.rs      # EpisodeRow, PlaybackStatus, DownloadStatus
│   ├── subscription.rs # Subscription
│   └── settings.rs     # deferred — not in this PR (constants live in defaults.rs)
├── model.rs            # Model (private; not serialized)
├── view_model.rs       # ViewModel, SubscriptionSummary
└── bin/
    └── codegen.rs      # unchanged
```

**`Cargo.toml` (workspace):** add `uuid = { version = "1", features = ["v4"] }`.
**`shared/Cargo.toml`:** add `uuid` from workspace.

### iOS (`iOS/`)

| File | Change |
|---|---|
| `iOS/project.yml` | Add GRDB Swift package dependency |
| `iOS/Pollux/core.swift` | Expand `processEffect` for `.storage` case |
| `iOS/Pollux/ContentView.swift` | Replace counter with stub library view |
| `iOS/Pollux/DatabaseManager.swift` | **New file** — GRDB pool, schema, migrations, execute() |

---

## Implementation Steps

### Step 1 — Domain types (Rust)

No settings struct. Settings become constants in a `shared/src/defaults.rs` file:
```rust
// TODO: factor each of these into the settings cascade
pub const SKIP_FORWARD_SECS: u32 = 30;
pub const SKIP_BACKWARD_SECS: u32 = 15;
pub const RESUME_REWIND_SECS: u32 = 3;
pub const REFRESH_INTERVAL_HOURS: u32 = 12;
```

Create `shared/src/domain/episode.rs`:
```rust
#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PlaybackStatus { Unplayed, InProgress, Played }

#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum DownloadStatus { NotDownloaded, Queued, Downloading, Downloaded, Failed, RemovedFromFeed }

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
pub struct Episode {
    pub id: String,                        // UUID (surrogate primary key, generated in core)
    pub feed_guid: String,                 // RSS <guid>; unique per subscription, not globally
    pub subscription_id: String,           // UUID of parent Subscription
    pub title: String,
    pub description: Option<String>,
    pub pub_date: Option<i64>,             // Unix timestamp seconds; nullable — RSS does not require <pubDate>
    pub duration_secs: Option<u32>,
    pub enclosure_url: String,            // remote audio URL from RSS feed
    pub artwork_url: Option<String>,
    pub playback_status: PlaybackStatus,
    pub playback_position_secs: Option<u32>,
    pub download_status: DownloadStatus,
    pub download_progress: Option<u8>,    // 0–100 while Downloading
    pub is_flagged: bool,
    pub file_size_bytes: Option<u64>,
    pub local_path: Option<String>,       // path to downloaded file on device
}
```

Create `shared/src/domain/subscription.rs`:
```rust
#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
pub struct Subscription {
    pub id: String,                      // UUID (generated in core)
    pub feed_url: String,                // unique; the RSS/Atom URI
    pub title: String,
    pub artwork_url: Option<String>,
    pub description: Option<String>,
    pub last_refreshed: Option<i64>,     // Unix timestamp
    pub created_at: i64,                 // Unix timestamp
}
```

No `settings.rs` in this pass.

### Step 2 — Storage capability (Rust)

Create `shared/src/capabilities/storage.rs`:
```rust
use crux_core::capability::Operation;

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
pub enum StorageOperation {
    UpsertSubscription(Subscription),
    GetSubscription { id: String },
    ListSubscriptions,
    DeleteSubscription { id: String },

    UpsertEpisode(Episode),
    GetEpisode { id: String },
    ListEpisodesBySubscription { subscription_id: String },
    GetEpisodeByFeedGuid { subscription_id: String, feed_guid: String },  // for RSS upsert dedup
    UpdatePlaybackStatus { episode_id: String, status: PlaybackStatus, position_secs: Option<u32> },
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
pub enum StorageResult {
    Success,
    Subscription(Subscription),
    Subscriptions(Vec<Subscription>),
    Episode(Episode),
    Episodes(Vec<Episode>),
    NotFound,
    Error(String),
}

impl Operation for StorageOperation {
    type Output = StorageResult;
}
```

### Step 3 — Effect, Event, Model, ViewModel (Rust, in `app.rs`)

**Effect enum** (add Storage variant):
```rust
#[effect(facet_typegen)]
#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
    Storage(StorageOperation),
}
```

**Event enum** (Phase 0 library flow):
```rust
pub enum Event {
    Init,
    SubscriptionsLoaded(StorageResult),
}
```

**Model** (in `model.rs`):
```rust
pub struct Model {
    pub subscriptions: Vec<Subscription>,
    pub loading: bool,
    pub error: Option<String>,
    // settings: GlobalSettings — deferred; settings.rs is out of scope for this PR
}
```

**ViewModel** (in `view_model.rs`):
```rust
#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct ViewModel {
    pub subscriptions: Vec<SubscriptionSummary>,
    pub loading: bool,
    pub error: Option<String>,
}

#[derive(Facet, Serialize, Deserialize, Clone)]
pub struct SubscriptionSummary {
    pub id: String,
    pub title: String,
    pub artwork_url: Option<String>,
}
```

**`update()` in `app.rs`:**
```rust
fn update(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Init => {
            model.loading = true;
            // Issue storage request; map result back as SubscriptionsLoaded
            // Exact crux_core 0.17 API to verify at implementation:
            // Command::request_from_shell(Effect::Storage(StorageOperation::ListSubscriptions))
            //     .then_send(Event::SubscriptionsLoaded)
            //     .and(render())
            todo!("verify crux_core 0.17 async Command API")
        }
        Event::SubscriptionsLoaded(result) => {
            model.loading = false;
            match result {
                StorageResult::Subscriptions(rows) => model.subscriptions = rows,
                StorageResult::Error(e) => model.error = Some(e),
                _ => {}
            }
            render()
        }
    }
}
```

> **Note on crux_core 0.17 async Command API:** The exact method for issuing an effect
> and mapping its response to an Event needs to be verified against crux_core 0.17 docs/source.
> Likely `Command::request_from_shell(effect).then_send(Event::Variant)` or the async closure
> pattern. Check `crux_core::Command` API before implementing Step 3.

### Step 4 — Tests (Rust)

Replace the counter tests with domain-model tests:
```rust
#[test]
fn init_issues_storage_request() { /* expect Storage effect */ }

#[test]
fn subscriptions_loaded_updates_view() {
    // Feed mock SubscriptionsLoaded event; check view has correct summaries
}

#[test]
fn empty_subscriptions_shows_empty_state() { /* ... */ }
```

### Step 5 — Run `make ios-build`

Regenerates Swift types in `iOS/generated/App/` and rebuilds the Xcode project.
This must pass before touching any Swift files.

### Step 6 — GRDB Swift package (iOS)

In `iOS/project.yml`, add:
```yaml
packages:
  Shared:
    path: ./generated/Shared
  App:
    path: ./generated/App
  GRDB:
    url: https://github.com/groue/GRDB.swift.git
    minorVersion: "6.0.0"

targets:
  Pollux:
    dependencies:
      - package: Shared
      - package: App
      - package: GRDB
        product: GRDB
```

### Step 7 — `DatabaseManager.swift` (new file)

```swift
import GRDB

actor DatabaseManager {
    private let db: DatabasePool

    init() throws {
        let path = /* app support directory */ + "/pollux.sqlite"
        db = try DatabasePool(path: path)
        try runMigrations()
    }

    private func runMigrations() throws {
        var migrator = DatabaseMigrator()
        migrator.registerMigration("v1_initial") { db in
            try db.create(table: "subscriptions") { t in
                t.column("id", .text).primaryKey()
                t.column("feed_url", .text).notNull().unique()  // one subscription per feed URL
                t.column("title", .text).notNull()
                t.column("artwork_url", .text)
                t.column("description", .text)
                t.column("last_refreshed", .integer)
                t.column("created_at", .integer).notNull()
            }
            try db.create(table: "episodes") { t in
                t.column("id", .text).primaryKey()             // surrogate UUID
                t.column("feed_guid", .text).notNull()         // RSS <guid>; unique per subscription
                t.column("subscription_id", .text).notNull()
                    .references("subscriptions", column: "id", onDelete: .cascade)
                t.column("title", .text).notNull()
                t.column("description", .text)
                t.column("pub_date", .integer)                 // nullable — RSS does not require <pubDate>
                t.column("duration_secs", .integer)
                t.column("enclosure_url", .text).notNull()
                t.column("artwork_url", .text)
                t.column("playback_status", .text).notNull().defaults(to: "Unplayed")
                t.column("playback_position_secs", .integer)
                t.column("download_status", .text).notNull().defaults(to: "NotDownloaded")
                t.column("download_progress", .integer)
                t.column("is_flagged", .boolean).notNull().defaults(to: false)
                t.column("file_size_bytes", .integer)
                t.column("local_path", .text)
                t.uniqueKey(["subscription_id", "feed_guid"])  // guid is only per-feed unique
            }
            try db.create(index: "episodes_subscription_id",
                          on: "episodes", columns: ["subscription_id"])
        }
        try migrator.migrate(db)
    }

    func execute(_ operation: StorageOperation) async throws -> StorageResult {
        switch operation {
        case .listSubscriptions:
            let rows = try await db.read { db in
                try Row.fetchAll(db, sql: "SELECT * FROM subscriptions ORDER BY title")
            }
            return .subscriptions(rows.map(Subscription.init))
        // ... all other operations
        case .upsertSubscription(let row):
            try await db.write { db in
                try db.execute(sql: """
                    INSERT INTO subscriptions (id, feed_url, title, artwork_url, description, last_refreshed, created_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(id) DO UPDATE SET title=excluded.title, artwork_url=excluded.artwork_url,
                    description=excluded.description, last_refreshed=excluded.last_refreshed
                    """,
                    arguments: [row.id, row.feedUrl, row.title, row.artworkUrl,
                                row.description, row.lastRefreshed, row.createdAt])
            }
            return .success
        // ... etc
        }
    }
}
```

### Step 8 — Expand `core.swift` processEffect

```swift
// Add DatabaseManager as a stored property
private let db: DatabaseManager

init() {
    // ...existing init...
    db = try DatabaseManager()  // throws; handle appropriately
}

func processEffect(_ request: Request) {
    switch request.effect {
    case .render:
        // existing
    case .storage(let op):
        Task {
            do {
                let result = try await db.execute(op.operation)
                let resultBytes = try result.bincodeSerialize()
                let effects = [UInt8](core.resolve(id: op.id, data: Data(resultBytes)))
                guard let newRequests: [Request] = try? .bincodeDeserialize(input: effects) else { return }
                for req in newRequests { processEffect(req) }
            } catch {
                let errResult = StorageResult.error(error.localizedDescription)
                guard let resultBytes = try? errResult.bincodeSerialize() else { return }
                let effects = [UInt8](core.resolve(id: op.id, data: Data(resultBytes)))
                guard let newRequests: [Request] = try? .bincodeDeserialize(input: effects) else { return }
                for req in newRequests { processEffect(req) }
            }
        }
    }
}
```

Also trigger `core.update(.init)` in the `Core.init()` after setting up the FFI, so the
library loads on startup.

### Step 9 — Stub `ContentView.swift`

Replace counter UI with a minimal library view:
```swift
struct ContentView: View {
    @ObservedObject var core: Core

    var body: some View {
        NavigationStack {
            Group {
                if core.view.loading {
                    ProgressView()
                } else if core.view.subscriptions.isEmpty {
                    Text("No podcasts yet")
                        .foregroundStyle(.secondary)
                } else {
                    List(core.view.subscriptions, id: \.id) { sub in
                        Text(sub.title)
                    }
                }
            }
            .navigationTitle("Library")
        }
    }
}
```

---

## Critical Files

| File | Role |
|---|---|
| `shared/src/app.rs` | App trait impl — `update()`, `view()` |
| `shared/src/capabilities/storage.rs` | `StorageOperation`, `StorageResult`, `Operation` impl |
| `shared/src/domain/episode.rs` | `Episode`, `PlaybackStatus`, `DownloadStatus` |
| `shared/src/domain/subscription.rs` | `Subscription` |
| `shared/src/defaults.rs` | Setting constants (all `// TODO: settings cascade`) |
| `shared/src/model.rs` | `Model` (private state) |
| `shared/src/view_model.rs` | `ViewModel`, `SubscriptionSummary` |
| `iOS/Pollux/DatabaseManager.swift` | GRDB schema, migrations, query executor |
| `iOS/Pollux/core.swift` | Effect dispatcher expansion |
| `iOS/Pollux/ContentView.swift` | Stub library view |
| `iOS/project.yml` | GRDB package dependency |

---

## Verification

1. `make test` — all Rust tests pass (new domain model tests, old counter tests removed)
2. `make rust-lint` — clippy clean
3. `make ios-build` — typegen + package + xcodegen succeed (new StorageOperation/StorageResult Swift types generated)
4. `make swift-lint` and `make swift-format-check` — clean
5. `make ios-sim` — app launches, shows empty library state ("No podcasts yet"), no crashes
6. Manual: confirm `Event::Init` triggers a `StorageOperation::ListSubscriptions` effect in Rust tests
7. Manual: confirm the `DatabaseManager` can be initialized without error (migrations run on first launch)
