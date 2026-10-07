# Player MVP (ROADMAP Phase 3)

## Context
Pollux (Crux: Rust core + SwiftUI shell) has no playback yet. Scaffolding exists: `Episode.playback_status/playback_position_secs`, `StorageOperation::UpdatePlaybackStatus` (implemented in Swift, unused by `app.rs`), downloads with relative `local_path`, and `.stubbed()` UI placeholders. Milestone: find an episode, play it, control it from the lock screen, resume where left off. Specs: `docs/features/player.md`, `mini-player.md`, `episode.md`, `settings.md`.

## Workspace
Implementation happens in a new worktree on branch `feat/player` (under `<repo>/.claude/worktrees/`, per global config), branched from `main`, not in this planning worktree. First step after approval: create it (EnterWorktree). No commits/pushes unless asked.

## Decisions (from user)
- Not downloaded: **stream from `enclosure_url` and start a download**; swap to the local file when the download finishes. Preroll the local item, zero-tolerance seek to the current time, then swap. Measure in sim; if audible, defer swap to next pause/seek.
- Full player: `fullScreenCover` from root; mini-player as bottom `safeAreaInset` on root (hidden when cover open or nothing active).
- **No auto-advance** in MVP. Playback stops at end; no queue/next/previous.
- Skip 30/15 and rewind 3s **hard-coded** in `shared/src/defaults.rs` behind one accessor (cascade/Settings later).
- Persist position every ~10s while playing + pause/seek/background/end, via existing `UpdatePlaybackStatus`. UI ticks (~1/s) are transient events, not persisted.
- Played = within **15s** of end (use AVPlayer duration if feed `duration_secs` missing); status -> Played, position -> 0. Replay of Played reverts to InProgress. Note a user story: played threshold user-configurable.
- Rewind 3s on any resume after pause, clamped at 0 (also cold-start restore); not on fresh start.
- Audio session `.playback` + `spokenAudio`; `UIBackgroundModes: audio` in `iOS/project.yml`; pause on headphone unplug; pause on interruption, auto-resume if `shouldResume`.
- Remote commands: play, pause, toggle, skip ±(30/15), change-playback-position. Now Playing info: title, feed, artwork, elapsed, rate.
- Errors: transient banner in player and mini-player (reuse `DownloadNoticeBanner` pattern, episode-scoped); stay paused. Missing local file -> fall back to streaming and reset `download_status`.
- Stubs: wire `EpisodeRow` play button; remove `EpisodeDetailView.playbackControls` stub row; keep bookmarks stub.

## Design
**Core (Rust, all logic/policy):**
- New `shared/src/capabilities/player.rs`: ops `Load{url, start_secs, autoplay}`, `Play`, `Pause`, `Seek{secs}`, `Stop`; outputs/events for position tick, duration known, ended, error, interruption. Add `Effect::Player` in `effect.rs`.
- `Model`: `active: Option<PlayContext{subscription_id, episode_id, position, duration, is_playing, source: Stream|Local}>`.
- Events: `PlayEpisode`, `TogglePlay`, `SkipForward/Back`, `SeekTo`, `PlayerTick`, `PlayerDuration`, `PlayerEnded`, `PlayerError`, `PlayerInterrupted`, `ClosePlayer`; reuse `DownloadFinished` to trigger source swap if that episode is active.
- `ViewModel`: `player: Option<PlayerView>` (episode/feed title, artwork url, position, duration, playing, source label "From: <subscription>", error notice). `EpisodeSummary`/loader must supply `local_path` and `enclosure_url` (or a resolved play source).
- Persist active context on cold start: new migration `v2_play_context` (single-row table: subscription_id, episode_id, position_secs) + Storage ops Save/Load/Clear; on `Started` restore paused with mini-player. Load episode by id (not just `model.episodes`).
- Skip clamps within [0, duration]; never crosses episodes; skip-forward into the final 15s ends the episode as Played.

**Shell (Swift):**
- `PlaybackManager` (`@MainActor`, new `iOS/Pollux/PlaybackManager.swift`) owns `AVPlayer`, periodic time observer, `AVAudioSession`, `MPNowPlayingInfoCenter`, `MPRemoteCommandCenter`, route/interruption observers. Handles `.player` in `Core.processEffect` (`iOS/Pollux/core.swift`).
- Expose local URL resolution: `DownloadManager.absoluteURL(for:)` is private; make a shared resolver (storage root + relative path). Digest filenames may lack an extension, so supply a type hint (`AVURLAssetOverrideMIMEType` / `AVURLAssetOutOfBandMIMETypeKey`) if sniffing fails.
- UI (themed via `themeColors`, a11y labels, adjustable-value scrubber): `MiniPlayerBar`, `PlayerScreen` (paged art/show-notes placeholder, dots, scrubber with tap-to-toggle remaining, skip labelled "+30"/"-15", play/pause, source context tappable -> dismiss cover and navigate to subscription, Hide, options menu with placeholders, episode menu). Chapter row hidden (no chapters yet).

## Docs
Update `docs/features/player.md` (resolved choices, 15s played tolerance), `ROADMAP.md`, `DATA_MODEL.md` PlayContext; add user story for configurable played threshold.

## Out of scope
Playlists, queue, chapters, speed, sleep timer, EQ, bookmarks, listening stats/PlayHistory, Settings screen, CarPlay specifics, background downloads.

## Verification
- `cargo test` for core: play/pause/skip clamps, rewind, played threshold, replay revert, source swap on `DownloadFinished`, persistence events (Crux effect assertions).
- Swift Testing: PlaybackManager helpers, URL resolution, DB migration v2 round-trip.
- `make check`, `make test`; build via `make ios-build` (regenerates types).
- Simulator: stream an undownloaded episode, watch the download finish and the swap (listen for gap), lock-screen controls, background/lock continues, kill + relaunch shows paused mini-player at saved position, airplane mode error banner.

## Status (after round 1)
Built and merged with main; `make check` and `make ios-test` pass. Where this differs from the plan above:
- The `play_context` table is part of the `v1_initial` baseline (per the schema-baseline policy in `docs/DATA_MODEL.md`), not a `v2` migration. It stores only `episode_id`; position lives on the episode row.
- The error banner is `NoticeBanner` (renamed on main from `DownloadNoticeBanner`).
- Resolved design choices are recorded in `docs/features/player.md` ("Decisions (MVP implementation)").

Verified in the simulator: streaming playback, skip, pause/resume with rewind, cold-start restore, "From:" navigation. **Not verified** (needs a device or a repro): lock-screen rendering, background audio, the stream-to-local swap gap, the error banner, audio interruptions, the AirPlay picker.

## Round 2: review follow-ups
Suggested order; each was its own commit. **Items 1 to 4 and both open questions are done**; 5 needs a device and 6 is deferred (below).
1. **Restore and played logic. Done.** As built: played is decided only when the listener leaves (natural end, an explicit pause in the final 15s, or starting another episode from the tail), never by a checkpoint; a seek that lands on the end finishes the episode; system pauses send a new `Interrupted` event that keeps the place and never finishes; every load has a session id and engine events from any other session are ignored; a played episode is not restored and its context is cleared. Original plan: do not restore an episode whose status is Played; clear the saved context instead (`on_context_loaded` in `player.rs`). Likely cause of "finished episode comes back at 0:00": a checkpoint in the last 15s marks it Played (position 0) but leaves the player active and the context saved, and only the engine's natural-end event clears it; reproduce first. Make "played" one decision (natural end, or an explicit seek into the tail) rather than a side effect of every checkpoint, covering: later ticks must not overwrite the reset position, episodes shorter than the tolerance, unknown duration, and a wrong feed duration before the engine reports the real one. Add a per-load session id to engine events so a stale event for the same episode (after a source swap or retry) cannot change state. Hide stays UI-only.
2. **Source-context abstraction. Done.** As built: the `EpisodeSource` enum (the name `DATA_MODEL.md` already planned), saved as `source_kind` + `source_id` in `play_context`; `PlayerSource` is now `MediaSource` and the engine's `Load` field is `media`. Original plan: add a small source context (subscription only for now) to `ActivePlayback` and `play_context` (kind + id, in the `v1_initial` baseline); "From:" and go-to-source read from it so playlists drop in later. Rename `PlayerSource` (stream/local) to `MediaSource` to avoid confusion. Keep `DATA_MODEL.md` PlayContext consistent.
3. **Show notes on the player's second page. Done.** The page is hidden, with its dots, when the episode has no show notes; timestamp detection is still unbuilt. Original plan: replace the "coming soon" placeholder with real show notes (reuse `ShowNotesText`); add the episode description to `PlayerView`. Update `docs/features/player.md` (currently says placeholder for MVP), `player-show-notes.md` and the ROADMAP.
4. **Player UI polish. Done.** Play/pause and the skip buttons (and the mini-player's play/pause) use the accent as filled glyphs; the secondary row stays in the text color; the artwork fills the width. Original plan: controls render black because they use the `text` token; use `accent` for primary controls and `text` for secondary (treatment TBD). Artwork fills the width at 1:1 (make `ArtworkView` flexible). A user-configurable player layout is a later feature; note it in `player.md`.
5. **Lock-screen controls are invisible but work. Open (needs a device).** Check on a real device first (may be a simulator quirk). If it persists: set `MPNowPlayingInfoPropertyMediaType` to audio and set the skip `preferredIntervals` before the first card update.
6. **Stream-to-local swap is audible (later). Open.** It seeks back to the last whole-second tick, so up to ~1s repeats, plus load time. Options: use the engine's exact time; preroll and seek the local item before swapping; or defer the swap to the next pause or seek.

Open questions, both resolved: the button treatment is accent-colored filled glyphs; `PlaybackManager` and `NowPlayingController` are now created on the first playback request (`startPlaybackIfNeeded` in `core.swift`) instead of in `Core.init`, so a background-refresh launch stays metadata-only. A restored, paused episode therefore appears on the lock screen only once it has been played.

## Round 3: branch review
A ten-point review of the whole branch, then a cross-check by a second reviewer. Each fix was its own commit with tests. Decisions are in `docs/features/player.md`.

Fixed:
1. **An interruption's end no longer resumes paused playback.** `Interrupted { resumable }` and `InterruptionEnded { should_resume }` replace the shell's unconditional `Play`; the core resumes only if the interruption is what paused it. Unplugging headphones is never resumed.
2. **A failed load no longer leaves the old audio playing.** The engine unloads the previous item when a load fails.
3. **A paused load no longer takes audio focus.** The audio session is claimed only for an autoplaying load and for `Play`.
4. **Only an unusable file discards a download.** `PlayerResult::MediaUnusable` and `PlayerMediaUnusable` trigger the stream fallback and re-download; other errors (another app holding the session, say) pause with a notice and leave the file alone.
6. **Now Playing is republished only on a content change or more than 2s of drift** (`NowPlayingPolicy`).
7. **Show notes are not parsed when the player opens.** Measured: the paged `TabView` builds the notes page lazily, so the parse happens on the first swipe (about 142 ms cold, 15 to 50 ms warm). No change made; it relies on `TabView`'s laziness.
8. **The spoken-time formatter is built once.**
9. **The storage root is looked up in one place** (`StorageRoot`).

From the cross-check:
- **Playback downloads its episode without its feed open.** `enqueue_download` also finds the active episode; `play()` starts or retries the download when the episode is streaming with no file coming. This fixes the missing-file fallback after a cold start.
- **A failed play-context save is retried** at each checkpoint until storage confirms it (`ActivePlayback::context_saved`, `PlayContextSaved`).

Partly done:
- **Per-tick render cost (5).** The player state is split from the episode list so ticks do not re-evaluate the list. In the simulator (debug core, 375 episodes) app CPU while playing with the list open fell from 5.4% to 4.1%, against a 1.0% floor. Still to do: measure on a device with a release build. If it is not fine, render less per tick. See "Known limitations and follow-ups" in `player.md`.
- **Shell tests (10).** The interruption and route-change mapping and session tagging are tested. Not covered: tick suppression while a seek is pending, and the natural-end and failure notifications (they need the engine's private item or real playback).

Still open:
- Lock-screen controls render invisibly (item 5 above); needs a device.
- The stream-to-local swap gap (item 6 above).
- Unverified without a device: background audio, the AirPlay picker, interruptions, the error banner, lock-screen rendering.
