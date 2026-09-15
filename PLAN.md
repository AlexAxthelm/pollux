# Download Manager (Phase 0)

## Context

Pollux's Phase 0 foundation is nearly complete: feed fetch → parse → save, the
library, subscribe flow, and subscription details with an episode list all work.
The `Episode` schema already carries `download_status`, `download_progress`,
`file_size_bytes`, and `local_path` (see `shared/src/domain/episode.rs`), and the
DB persists them (`iOS/Pollux/DatabaseManager.swift`), but **nothing drives those
fields** — the download control in `EpisodeRow.swift` is an inert DEBUG-tinted
stub. This change implements the download manager: a serial queue in the core, a
new `Download` shell capability that actually fetches and stores audio files, and
wiring the episode-row control so a user can download an episode, watch its status,
and delete the local file.

Scope decisions (confirmed with the user):
- **Progress: status-only for the first pass**, but the capability and shell are
  structured so live 0–100% progress can be added *before this PR merges* without
  a rewrite (a `Progress` result variant + a URLSession delegate seam are stubbed
  in from the start).
- **Storage soft-cap: deferred.** The user-configurable cap depends on the Settings
  screen (not built yet). We build the pre-download check as a seam that defaults to
  *unlimited*, and rely on real device-full write errors for fail-on-full. The
  configurable cap lands when Settings does.
- **Dedicated Downloads/Storage screens are Phase 5**, not this change. This change
  makes the per-episode control functional; a queue screen is a follow-up.

Roadmap item: Phase 0 — "Download manager: queue, serial downloads for MVP, storage
limit check before download, fail-on-full behavior" (`docs/ROADMAP.md`).
Feature specs: `docs/features/storage.md`, `docs/features/episode.md`.

---

## Approach

Follow the existing capability pattern exactly (mirror `capabilities/http.rs` and
its shell handler in `core.swift`). Downloads are shell I/O; the core only tracks
queue + status and persists transitions through the existing `Storage` capability.

### 1. New `Download` capability (Rust)

`shared/src/capabilities/download.rs` (new), registered in
`shared/src/capabilities/mod.rs`:

```rust
pub enum DownloadOperation {
    Download { episode_id: String, url: String },
    Delete   { local_path: String },
}

pub enum DownloadResult {
    // Present from day one so adding live progress later is additive, not a
    // signature change. Unused by the core in the first pass.
    Progress { episode_id: String, percent: u8 },
    Completed { local_path: String, size_bytes: u64 },
    Deleted,
    Error(String),
}
impl Operation for DownloadOperation { type Output = DownloadResult; }
```

Add `Download(DownloadOperation)` to the `Effect` enum in `shared/src/effect.rs`
(same shape as the existing `Http`/`Storage` arms).

### 2. Core: queue + serial execution (`shared/src/app.rs`, `model.rs`)

- **Model** (`model.rs`): add `download_queue: Vec<String>` (episode ids waiting)
  and `downloading: Option<String>` (the one in flight — enforces *serial*).
- **Events** (`app.rs`): `DownloadEpisode(String)`, `DeleteDownload(String)`,
  `DownloadFinished { episode_id, result: Box<DownloadResult> }`.
- **`DownloadEpisode`**: the pre-download **limit-check seam** lives here
  (`fn download_allowed(model) -> bool`, returns `true` today = unlimited). If the
  episode is already downloading/queued, no-op. Otherwise mark it `Queued`
  (persist via `StorageOperation::UpsertEpisode` or a narrower status update — see
  note below), push to queue, and call `maybe_start_next`.
- **`maybe_start_next`**: if `downloading.is_none()` and the queue is non-empty,
  pop the front, set `downloading`, mark it `Downloading`, and emit a
  `DownloadOperation::Download` request whose resolution sends
  `DownloadFinished`.
- **`DownloadFinished`**: on `Completed`, set status `Downloaded` + store
  `local_path`/`file_size_bytes`; on `Error`, set status `Failed`. Either way clear
  `downloading` and call `maybe_start_next` (serial drain). Persist the transition.
- **`DeleteDownload`**: emit `DownloadOperation::Delete { local_path }`, and on the
  result reset the row to `NotDownloaded` with `local_path`/`size`/`progress`
  cleared, persisted.
- Reuse the existing selection-correlation discipline already in
  `EpisodesLoaded`: after any status change to an episode of the currently-selected
  feed, the details list must reflect it. **This is the exact hazard flagged at
  `app.rs:107` / `subscription.md:73`** — `SelectSubscription` is idempotent, so a
  status write must update `model.episodes` in place (preferred) rather than relying
  on re-selection. Update the matching entry in `model.episodes` directly in the
  event handlers so `view()` reprojects it.

**Storage op note:** there is no targeted "update download status" storage op yet
(only `UpdatePlaybackStatus`). Add `StorageOperation::UpdateDownloadState { episode_id,
status, local_path, size_bytes, progress }` mirroring `UpdatePlaybackStatus`, and
implement it in `DatabaseManager.swift` (a single `UPDATE`). Prefer this over
re-upserting the whole episode.

### 3. Shell: Download handler (`iOS/Pollux/core.swift` + new file)

- Add a `case let .download(operation)` arm in `processEffect` (mirror `.http`).
- New `iOS/Pollux/DownloadManager.swift`: performs the URLSession download to a
  file in Application Support (reuse the `DatabaseManager` directory convention —
  a `Downloads/` subdir; store a **relative** `local_path` so it survives container
  path changes, resolving to absolute at read time), returns
  `.completed(localPath:sizeBytes:)` or `.error`. `Delete` removes the file
  (idempotent if already gone). Structure the download with a
  `URLSessionDownloadDelegate` so the `didWriteData` progress callback is the single
  seam where live-% reporting gets added before merge (first pass: ignore it).
- Add `resolveAndDispatch(_:result: DownloadResult)` overload (mirror the Http one).
- Serial execution is enforced by the **core** (`downloading` guard), so the shell
  handler stays a simple one-shot per request.

### 4. UI wiring (`iOS/Pollux/EpisodeRow.swift`, `EpisodeDetailView.swift`)

- Replace the stubbed download placeholder in `placeholderControls` with a real
  control driven by `episode.downloadStatus`:
  - `notDownloaded` → download button → `core.update(.downloadEpisode(id))`
  - `queued` / `downloading` → progress/indeterminate indicator (the existing
    `downloadBadge` already renders these states; keep the badge, make the control
    reflect activity)
  - `downloaded` → "downloaded" affordance with a delete action →
    `core.update(.deleteDownload(id))`
  - `failed` → retry → `core.update(.downloadEpisode(id))`
- `EpisodeRow` takes only `episode: EpisodeSummary` today; thread the `core` (or an
  action closure) in the same way `SubscriptionDetailScreen` already passes `core`.
- Keep the play/more-actions stubs as-is (out of scope).
- No standalone Downloads screen in this change (Phase 5).

### 5. Codegen + tests

- Regenerate Swift types after the Rust changes: `make ios-build` (runs `typegen`
  + `package` + `generate-project`). Required whenever `Event`/`Effect`/capability
  enums change.
- **Rust unit tests** in `app.rs` (follow the existing `mod tests` style): enqueue
  → `Downloading` + one `Download` effect; a second enqueue while one is in flight
  stays `Queued` and emits no new download effect (serial); `Completed` transitions
  to `Downloaded`, stores path/size, and starts the next queued item; `Error`
  transitions to `Failed` and still drains the queue; delete resets to
  `NotDownloaded`; a status change to a selected-feed episode is reflected in
  `view().subscription_detail` without re-selection.
- **Swift tests** (`iOS/PolluxTests/`): a `DatabaseManagerTests` case for the new
  `UpdateDownloadState` op (round-trips status + path + size). A focused
  `DownloadManager` test for the delete path is optional if network mocking is
  heavy — keep to what's cheap.

---

## Critical files

- `shared/src/capabilities/download.rs` — **new** capability
- `shared/src/capabilities/mod.rs` — register it
- `shared/src/capabilities/storage.rs` — add `UpdateDownloadState`
- `shared/src/effect.rs` — add `Download` effect arm
- `shared/src/model.rs` — `download_queue`, `downloading`
- `shared/src/app.rs` — events, queue drain, in-place episode status updates, tests
- `iOS/Pollux/core.swift` — `.download` effect arm + resolve overload
- `iOS/Pollux/DownloadManager.swift` — **new** shell downloader (delegate seam)
- `iOS/Pollux/DatabaseManager.swift` — implement `UpdateDownloadState`
- `iOS/Pollux/EpisodeRow.swift` / `EpisodeDetailView.swift` — functional control
- `iOS/PolluxTests/DatabaseManagerTests.swift` — new-op coverage

Reuse: the `HttpOperation`/`updatePlaybackStatus` patterns are the templates for
the capability and the storage op respectively; `DebugStyle`/`.stubbed()` is what's
being *removed* from the download control.

---

## Verification

1. `make test` — Rust core tests (`cargo test`) + iOS tests. New queue/status tests
   must pass.
2. `make check` — `cargo check/clippy -D warnings/fmt --check/--locked`, swiftlint
   `--strict`, swiftformat `--lint`. CI runs the same; clippy warnings and the
   admin forbidden-pattern scan (no force-unwrap/try, no `unsafe`) will fail the
   build otherwise.
3. `make ios-build` — confirm codegen succeeds and the generated Swift `Event`/
   `Request` types include the new download variants.
4. Manual on simulator (`make ios-sim`, or the iOS Simulator MCP — pick the newest
   `Pollux.app` by mtime per the DerivedData memory note): subscribe to a real feed,
   open a subscription, tap download on an episode → status goes
   Queued → Downloading → Downloaded; queue a second while the first runs → it
   waits (serial); delete a download → returns to Not downloaded; kill and relaunch
   → downloaded episodes still read as Downloaded (persisted). Use `NSLog` (not
   `print`) for any temporary logging.

## Explicit follow-ups (not in this change)

- Live 0–100% progress via the stubbed URLSession delegate seam + `Progress` result
  variant (**intended before this PR merges**, per the user).
- User-configurable storage soft-cap + the pre-download limit check made real (needs
  the Settings screen).
- Dedicated Downloads queue screen and Storage usage breakdown (Phase 5).
- Cold-start re-enqueue of episodes left in `Queued` status.
