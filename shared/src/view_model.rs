use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::domain::{DownloadStatus, EpisodeSortOrder, PlaybackStatus};
use crate::theme::ThemeView;

#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct ViewModel {
    pub library: LibraryView,
    pub subscription_detail: SubscriptionDetailView,
    /// Active theme, resolved by the shell into platform colors. Present on every
    /// render so the shell can apply it globally without a separate query.
    pub theme: ThemeView,
}

#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct LibraryView {
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
    pub download_progress: Option<u8>,
}
