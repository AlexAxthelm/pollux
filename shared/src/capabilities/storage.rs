use crux_core::capability::Operation;
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::domain::{DownloadStatus, Episode, PlaybackStatus, Subscription};

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum StorageOperation {
    UpsertSubscription(Subscription),
    GetSubscription {
        id: String,
    },
    ListSubscriptions,
    DeleteSubscription {
        id: String,
    },
    UpsertEpisode(Episode),
    GetEpisode {
        id: String,
    },
    ListEpisodesBySubscription {
        subscription_id: String,
    },
    /// Every episode currently marked `Downloading` or `Queued`, across all feeds.
    /// Used at launch to rebuild the download queue for downloads interrupted by a
    /// previous quit.
    ListPendingDownloads,
    GetEpisodeByFeedGuid {
        subscription_id: String,
        feed_guid: String,
    },
    UpdatePlaybackStatus {
        episode_id: String,
        status: PlaybackStatus,
        position_secs: Option<u32>,
    },
    /// Persists a download-state transition (status plus the file metadata that
    /// comes with it). `local_path`/`size_bytes` are cleared to NULL when `None` —
    /// e.g. on delete or failure — so the row never keeps a stale path for a file
    /// that is no longer there. (Live byte progress is transient and never stored.)
    UpdateDownloadState {
        episode_id: String,
        status: DownloadStatus,
        local_path: Option<String>,
        size_bytes: Option<u64>,
    },
    /// Records the outcome of a refresh that did not produce a new feed body (304,
    /// 429, or a failure). `last_refreshed` is left unchanged when `None`; the error
    /// and retry-after columns are written verbatim (NULL when `None`). A moved
    /// `last_refreshed` (a 304) also confirms episodes already missing from the feed:
    /// those whose absence has lasted the grace period are flagged, and the count comes
    /// back as `StorageResult::EpisodesRemoved`.
    UpdateRefreshState {
        subscription_id: String,
        last_refreshed: Option<i64>,
        last_refresh_error: Option<String>,
        retry_after_until: Option<i64>,
    },
    UpsertFeedWithEpisodes {
        subscription: Subscription,
        episodes: Vec<Episode>,
    },
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum StorageResult {
    Success,
    Subscription(Subscription),
    Subscriptions(Vec<Subscription>),
    Episode(Episode),
    Episodes(Vec<Episode>),
    /// Answer to `UpdateRefreshState`: how many episodes that write flagged
    /// `RemovedFromFeed` (a 304 confirms pending absences; nothing else flags any).
    EpisodesRemoved(u64),
    NotFound,
    Error(String),
}

impl Operation for StorageOperation {
    type Output = StorageResult;
}
