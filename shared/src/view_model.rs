use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::domain::{DownloadStatus, EpisodeSortOrder, EpisodeSource, PlaybackStatus};
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
    /// The global settings as the Settings screen shows them (see `SettingsView`).
    pub settings: SettingsView,
}

/// The settings page for one level, one entry per setting. The global page and a subscription's
/// page have this shape, and a playlist's will too. `reset_value` holds the value inherited from
/// the level above, which is the app default at the global level.
#[derive(Facet, Serialize, Deserialize, Clone)]
pub struct SettingsView {
    pub skip_forward_secs: SettingValue,
    pub skip_back_secs: SettingValue,
    pub resume_rewind_secs: SettingValue,
}

impl Default for SettingsView {
    /// Every setting at its app default, which is what an empty `Settings` projects to.
    fn default() -> Self {
        crate::settings::settings_view(
            &crate::settings::Settings::default(),
            &crate::settings::ResolvedSettings::default(),
        )
    }
}

/// One setting as the page shows it: the value in effect, and what it would be if reset.
/// The shell writes "(default)" when `is_set` is false, and offers a reset when it is true.
#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingValue {
    /// The value in effect at this level.
    pub value: u32,
    /// The value this setting takes when it isn't set here (the app default at the global
    /// level). Resetting the setting makes `value` equal to this. Not called `default`, which
    /// is a Swift keyword and so can't be a field in the generated types.
    pub reset_value: u32,
    /// The user has set this value at this level, so it is shown as customised. True even
    /// when `value == reset_value`: it was chosen on purpose, and it stays if the default
    /// changes (settings-cascade).
    pub is_set: bool,
}

/// The active episode's transport state, present whenever the mini-player should show
/// (playing or paused) and absent when nothing is active. The shell derives the
/// full-player and lock-screen presentation from this alone.
#[derive(Facet, Serialize, Deserialize, Clone)]
pub struct PlayerView {
    pub episode_id: String,
    pub episode_title: String,
    /// The episode's own feed (the Now Playing "artist"), distinct from `source`.
    pub feed_title: String,
    /// What playback was started from; the "From:" row names `source_title` and
    /// navigates back to `source`.
    pub source: EpisodeSource,
    pub source_title: String,
    /// Episode art, falling back to the feed's; the shell supplies the placeholder.
    pub artwork_url: Option<String>,
    pub position_secs: u32,
    pub duration_secs: Option<u32>,
    pub is_playing: bool,
    /// True while playing from the network rather than a downloaded file.
    pub is_streaming: bool,
    pub skip_forward_secs: u32,
    pub skip_back_secs: u32,
    /// The episode's show notes (raw HTML from the feed) for the player's show-notes
    /// page, or `None` when it has none (the page is then not shown). Plain-text
    /// `description_text` is the fallback shown until the shell has rendered the HTML.
    pub description: Option<String>,
    pub description_text: Option<String>,
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
    /// The selected subscription's settings page. A value it hasn't set shows what it
    /// inherits from the global settings, and a reset gives that value. Default when no
    /// subscription is selected.
    pub settings: SettingsView,
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
