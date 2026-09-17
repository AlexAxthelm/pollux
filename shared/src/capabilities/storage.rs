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
    /// comes with it). `local_path`/`size_bytes`/`progress` are cleared to NULL
    /// when `None` — e.g. on delete or failure — so the row never keeps a stale
    /// path for a file that is no longer there.
    UpdateDownloadState {
        episode_id: String,
        status: DownloadStatus,
        local_path: Option<String>,
        size_bytes: Option<u64>,
        progress: Option<u8>,
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
    NotFound,
    Error(String),
}

impl Operation for StorageOperation {
    type Output = StorageResult;
}
