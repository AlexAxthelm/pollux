# Feed refresh

## Context
Pollux (Crux app: Rust core in `shared/`, SwiftUI shell in `iOS/Pollux/`, GRDB/SQLite) can subscribe to a feed but never re-fetch it. Refresh is specced in `docs/features/subscription.md:53-90` (metadata-only, conditional GET, 200/304/429, 12h interval, manual + background). No refresh UI, no BGTaskScheduler, no etag storage exists. Reuse the fetch → `parse_feed` → `UpsertFeedWithEpisodes` path, whose upserts already preserve ids and playback/download state.

## Decisions (from user)
- Four steps, each commit-sized. **Changes are left in the working tree; the user commits.**
- Full ETag + Last-Modified conditional GET; handle 304 and 429/Retry-After (persist next-allowed-time per feed).
- Episodes that drop out of a feed → mark `DownloadStatus::RemovedFromFeed` (rows and files kept).
- Failures: per-row indicator + persisted last error.
- Interval: hardcoded `defaults::REFRESH_INTERVAL_HOURS` (12h); settings/per-feed override deferred.
- Refresh all: serial, one feed at a time.
- Background: `BGAppRefreshTask`, refreshes all due feeds, no downloads.

## Step 0 (foundation, ships with step 1)
- `shared/src/capabilities/http.rs`: add request headers (`If-None-Match`, `If-Modified-Since`) to `FetchFeed`; add response `etag`, `last_modified`, `retry_after` to `HttpResult::Response`. Run `make typegen`.
- `iOS/Pollux/core.swift:131-149`: send/read those headers in `fetchFeed`.
- `Subscription` (`shared/src/domain/subscription.rs`) + new GRDB migration `v2_refresh` in `DatabaseManager.swift`: `etag`, `last_modified`, `last_refresh_error`, `retry_after_until`. Update `DatabaseManagerRows.swift` mapping.
- Separate refresh state in `model.rs` (per-subscription in-flight set, errors); do NOT reuse `loading`/`error`, since `ContentView.swift:60-76` and `SubscribeFlow.swift` infer subscribe success from them.
- New events (`RefreshSubscription(id)`, `RefreshFetched`, `RefreshAll`, ...) in `shared/src/app.rs`, with a serial refresh queue modelled on the download queue helpers (`app.rs:565-668`).
- Core handling: 200 → parse, upsert, store validators, clear error; 304 → touch `last_refreshed` only; 429 → store `retry_after_until`; other/network → store `last_refresh_error`.
- Upsert changes: fix `file_size_bytes` clobbering (`DatabaseManagerRows.swift:37-46`); mark missing episodes `RemovedFromFeed` in the same transaction (`DatabaseManager.swift:154-176`).
- After a refresh of the selected subscription, explicitly reload episodes (`app.rs:148-159`).

## Step 1: details-page refresh
`.refreshable` on the list in `SubscriptionDetailScreen.swift:79` dispatching `RefreshSubscription`; refresh spinner via the new state.

## Step 2: Refresh all
Library toolbar button (`ContentView.swift`), progress/disabled state, per-row failure indicator (add `last_refresh_error`/flag to `SubscriptionSummary` in `view_model.rs`).

## Step 3: Foreground auto-refresh
On launch/scene-active, enqueue subscriptions where `now - last_refreshed >= 12h` and `retry_after_until` has passed.

## Step 4: Background refresh
`project.yml`: `BGTaskSchedulerPermittedIdentifiers` + `UIBackgroundModes: fetch`; register `BGAppRefreshTask` in `Pollux.swift`, schedule ~12h `earliestBeginDate`, reschedule each run, run the same due-feed queue, honor expiration.

## Tests
- Rust (`make rust-test`): 200/304/429/error transitions, RemovedFromFeed marking, queue order, no touching `loading`/`error`.
- Swift (`make ios-test`): migration v2, file-size preservation (extend `DatabaseManagerTests.swift:406-478`), header handling with `MockURLProtocol` (pattern in `DownloadManagerTests.swift:13`).
- Prefer make targets: `make typegen`, `make package`, `make generate-project`, `make check`, `make ios-build`.

## Verification
`make check` + `make ios-test`, then `make ios-sim` against a real feed: pull-to-refresh (observe 304 on second pull), Refresh all, failure row on a bad URL, relaunch after >12h; background via Xcode "Simulate Background Fetch".

## Docs
Update `docs/features/subscription.md`, `docs/ROADMAP.md`, `docs/DATA_MODEL.md` as behavior lands.
