use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::domain::{DownloadStatus, EpisodeSortOrder, PlaybackStatus};
use crate::theme::ThemeView;

#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct ViewModel {
    pub library: LibraryView,
    pub subscription_detail: SubscriptionDetailView,
    /// The active player, if any (see `PlayerView`).
    pub player: Option<PlayerView>,
    /// Active theme, resolved by the shell into platform colors. Present on every
    /// render so the shell can apply it globally without a separate query.
    pub theme: ThemeView,
}

/// The active episode's transport state, present whenever the mini-player should show
/// (playing or paused) and absent when nothing is active. The shell derives the
/// full-player and lock-screen presentation from this alone.
#[derive(Facet, Serialize, Deserialize, Clone)]
pub struct PlayerView {
    pub episode_id: String,
    pub subscription_id: String,
    pub episode_title: String,
    pub feed_title: String,
    /// Episode art, falling back to the feed's; the shell supplies the placeholder.
    pub artwork_url: Option<String>,
    pub position_secs: u32,
    pub duration_secs: Option<u32>,
    pub is_playing: bool,
    /// True while playing from the network rather than a downloaded file.
    pub is_streaming: bool,
    pub skip_forward_secs: u32,
    pub skip_back_secs: u32,
    /// Why playback failed, shown as a transient banner; playback stays paused.
    pub error: Option<String>,
}

#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct LibraryView {
    pub subscriptions: Vec<SubscriptionSummary>,
    pub loading: bool,
    pub error: Option<String>,
    /// Any feed is queued for or being refreshed (drives the "Refresh all" button).
    pub refreshing: bool,
}

#[derive(Facet, Serialize, Deserialize, Clone)]
pub struct SubscriptionSummary {
    pub id: String,
    pub title: String,
    pub artwork_url: Option<String>,
    /// Queued for, or in the middle of, a refresh.
    pub refreshing: bool,
    /// Why the last refresh failed, for the row's failure indicator. Absent after a
    /// successful refresh.
    pub refresh_error: Option<String>,
}

/// The selected subscription's episode list, shown on the details page. Empty by
/// default (no subscription selected); populated after `SelectSubscription`.
#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct SubscriptionDetailView {
    pub subscription_id: Option<String>,
    pub title: String,
    pub artwork_url: Option<String>,
    pub episodes: Vec<EpisodeSummary>,
    pub sort_order: EpisodeSortOrder,
    pub loading: bool,
    pub error: Option<String>,
    /// A transient, non-blocking notice for a failed download *operation* (a
    /// persistence write or file removal that didn't take). Shown as a banner that
    /// leaves the episode list and its controls intact, unlike `error`. Tagged with
    /// the episode it concerns so an episode's detail page shows it only when it's
    /// about that episode, while the list shows it for any.
    pub download_notice: Option<DownloadNotice>,
    /// The open feed is queued for or being refreshed.
    pub refreshing: bool,
    /// A non-blocking warning that the list below may be out of date, because reloading
    /// it after a refresh failed. Shown as a banner above the list, which stays in place
    /// (unlike `error`, which replaces it).
    pub list_notice: Option<String>,
}

/// A failed-download-operation notice, paired with the episode it's about. Transient
/// and non-blocking (see `SubscriptionDetailView::download_notice`).
#[derive(Facet, Serialize, Deserialize, Clone)]
pub struct DownloadNotice {
    pub episode_id: String,
    pub message: String,
}

/// Read-only projection of an `Episode` for display. Dates and durations stay raw
/// (`i64`/`u32`) so the shell can format them locale-aware; the core owns which
/// data ships and its ordering.
#[derive(Facet, Serialize, Deserialize, Clone)]
pub struct EpisodeSummary {
    pub id: String,
    pub title: String,
    /// Raw description (may contain HTML); the detail page renders it as rich text.
    pub description: Option<String>,
    /// Plain-text description for compact previews (HTML stripped, core-side).
    pub description_text: Option<String>,
    pub pub_date: Option<i64>,
    pub duration_secs: Option<u32>,
    pub artwork_url: Option<String>,
    pub playback_status: PlaybackStatus,
    pub playback_position_secs: Option<u32>,
    pub download_status: DownloadStatus,
    /// Live download progress, present only for the episode currently downloading.
    /// `received`/`total` are byte counts; `total` is absent when the server didn't
    /// report a size, in which case the shell shows an indeterminate indicator.
    /// Transient and never persisted (see `Model::active_download_progress`).
    pub download_received_bytes: Option<u64>,
    pub download_total_bytes: Option<u64>,
    /// Why the download failed, present only for a `Failed` episode. Lets the shell
    /// show the specific reason (disk full, HTTP status, …) beside Retry rather than
    /// a generic message. Transient (see `Model::download_errors`).
    pub download_error: Option<String>,
}
