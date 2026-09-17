use crate::domain::{Episode, EpisodeSortOrder, Subscription};

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
    // (`downloading`), the rest wait in `download_queue` (front = next up). Both
    // hold episode ids. The queue is in-memory only today; episodes left in the
    // `Queued` status across a cold start are not yet re-enqueued (see ROADMAP
    // follow-ups).
    pub download_queue: Vec<String>,
    pub downloading: Option<String>,

    // Live progress of the in-flight download (the one in `downloading`). Transient
    // and never persisted: a partial download can't resume across a restart, so a
    // stored percentage would only be stale. Cleared when a download ends or the
    // next one starts.
    pub active_download_progress: Option<DownloadProgress>,
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
