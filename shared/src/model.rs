use std::collections::HashMap;

use crate::domain::{Episode, EpisodeSortOrder, Subscription};
use crate::theme::{ThemeId, ThemeMode};

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

    // Download manager. Serial for MVP: at most one episode downloads at a time
    // (`downloading` holds its id), the rest wait in `download_queue` (front = next
    // up). Queue entries carry the enclosure URL so a download can start without its
    // feed's episodes being loaded — this is what lets `Started` re-enqueue downloads
    // interrupted by a previous quit (see `PendingDownloadsLoaded`). In-memory only:
    // the queue itself is rebuilt from the DB at launch.
    pub download_queue: Vec<QueuedDownload>,
    pub downloading: Option<String>,

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

    // Active theme selection. Defaults (System / FollowSystem) reproduce the OS's
    // native appearance. Hard-coded for now — no UI changes it until the Settings
    // appearance section lands and drives `Event::SetTheme`.
    pub theme_id: ThemeId,
    pub theme_mode: ThemeMode,
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
