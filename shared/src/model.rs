use std::collections::HashMap;

use crate::capabilities::player::MediaSource;
use crate::domain::{Episode, EpisodeSortOrder, EpisodeSource, Subscription};
use crate::theme::{ThemeId, ThemeMode};
use crate::view_model::DownloadNotice;

#[derive(Default)]
pub struct Model {
    pub subscriptions: Vec<Subscription>,
    pub loading: bool,
    pub error: Option<String>,

    // Subscription details page. Kept separate from the library `loading`/`error`
    // above so opening a feed's episode list never clobbers the Library state.
    pub selected_subscription: Option<Subscription>,
    pub episodes: Vec<Episode>,
    pub episode_sort: EpisodeSortOrder,
    pub detail_loading: bool,
    pub detail_error: Option<String>,
    // A non-blocking warning that the episode list on screen may be out of date: a
    // reload (after a refresh) failed while a list was already showing. Unlike
    // `detail_error`, which replaces the whole list, this leaves the list in place and
    // is shown as a banner. While it is set, re-entering the feed and a 304 refresh of it
    // both retry the reload in place; it is cleared by a successful load or a feed switch.
    pub list_notice: Option<String>,

    // Download manager. Serial for MVP: at most one episode downloads at a time
    // (`downloading` holds its id), the rest wait in `download_queue` (front = next
    // up). Queue entries carry the enclosure URL so a download can start without its
    // feed's episodes being loaded — this is what lets `Started` re-enqueue downloads
    // interrupted by a previous quit (see `PendingDownloadsLoaded`). In-memory only:
    // the queue itself is rebuilt from the DB at launch.
    pub download_queue: Vec<QueuedDownload>,
    pub downloading: Option<String>,
    // Set once the first foreground activation has asked storage for downloads a
    // previous session left in flight, so later activations don't repeat the request.
    pub pending_downloads_requested: bool,

    // Live progress of the in-flight download (the one in `downloading`). Transient
    // and never persisted: a partial download can't resume across a restart, so a
    // stored percentage would only be stale. Cleared when a download ends or the
    // next one starts.
    pub active_download_progress: Option<DownloadProgress>,

    // Why a download failed, keyed by episode id, so the shell's specific reason
    // (disk full, HTTP status, filesystem error) can be shown beside Retry instead
    // of a generic "failed". In-memory only (like progress) and cleared as soon as
    // the episode leaves the Failed state; the persisted status is enough to know it
    // failed across a restart.
    pub download_errors: HashMap<String, String>,

    // A transient, non-blocking notice for a download *operation* that failed without
    // changing the episode's state — a persistence write that didn't commit, or a file
    // that couldn't be removed. Tagged with the failing episode so the detail page can
    // scope it to that episode. Shown as a banner on the details page (NOT through
    // `detail_error`, which replaces the whole episode list). Cleared when the user
    // starts another download action or switches feeds.
    pub download_notice: Option<DownloadNotice>,

    // The episode the player is on (playing or paused), if any. Survives restarts via
    // the stored play context; see `player.rs`.
    pub active_playback: Option<ActivePlayback>,
    // Last session id handed to the engine (see `player.rs`). Only ever increases, so an
    // id is never reused across episodes or reloads.
    pub player_sessions: u32,

    // Feed refresh. Serial like downloads: at most one feed is fetched at a time
    // (`refreshing` holds its subscription id), the rest wait in `refresh_queue`
    // (front = next up). Kept apart from the library `loading`/`error`, which the
    // subscribe flow reads to detect success. In-memory only; per-feed outcomes
    // (error, retry-after) live on the `Subscription` and are persisted.
    pub refresh_queue: Vec<String>,
    pub refreshing: Option<String>,
    // Auto-refresh can be requested (the app became active) before the first library
    // load lands, when there is nothing to judge staleness against. The request is held
    // here and honoured as soon as the subscriptions arrive.
    pub subscriptions_loaded: bool,
    pub auto_refresh_pending: bool,

    // Active theme selection. Defaults (System / FollowSystem) reproduce the OS's
    // native appearance. Hard-coded for now — no UI changes it until the Settings
    // appearance section lands and drives `Event::SetTheme`.
    pub theme_id: ThemeId,
    pub theme_mode: ThemeMode,
}

/// The player's current episode and transport state. Holds its own `Episode` copy so
/// playback doesn't depend on `model.episodes` (which only holds the feed on screen,
/// and is empty after a cold-start restore).
#[derive(Clone, Debug)]
pub struct ActivePlayback {
    pub episode: Episode,
    /// The engine load this state belongs to; engine news for any other is stale.
    pub session: u32,
    pub position_secs: u32,
    /// The engine's duration, authoritative over the feed's `episode.duration_secs`.
    pub duration_secs: Option<u32>,
    pub is_playing: bool,
    /// What the engine reads: the stream or the downloaded file.
    pub media: MediaSource,
    /// What playback was started from (a subscription today, a playlist later); named
    /// by the player's "From:" row and navigated back to from it.
    pub source: EpisodeSource,
    /// Whether the shell's engine currently has this episode loaded. False right after
    /// a cold-start restore (and after an error), so the next Play issues a `Load`.
    pub loaded: bool,
    /// Position at the last persisted checkpoint, to space periodic writes.
    pub last_checkpoint_secs: u32,
    /// Set when an audio-session interruption paused playback that was running, so the
    /// end of that interruption may resume it. Cleared by any explicit play or seek, and
    /// never set when playback was already paused (the listener's choice stands).
    pub resume_after_interruption: bool,
    /// Whether this episode is known to be saved as the play context a relaunch restores.
    /// False from the moment playback starts until storage confirms the write; while it is
    /// false every checkpoint re-sends it, so one failed write can't leave a relaunch
    /// restoring the wrong episode (or none).
    pub context_saved: bool,
    pub error: Option<String>,
}

/// An episode waiting to be downloaded. Carries the enclosure URL so the download
/// can be issued from the queue alone, without the episode being loaded in
/// `model.episodes`.
#[derive(Clone, Debug)]
pub struct QueuedDownload {
    pub episode_id: String,
    pub url: String,
}

/// Byte progress of the currently-downloading episode. `total_bytes` is optional
/// because a server may omit `Content-Length`; when it's `None` the shell can't
/// know the size, so the UI shows an indeterminate indicator rather than a bar.
#[derive(Clone, Debug)]
pub struct DownloadProgress {
    pub episode_id: String,
    pub received_bytes: u64,
    pub total_bytes: Option<u64>,
}
