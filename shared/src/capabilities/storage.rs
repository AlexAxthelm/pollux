use crux_core::capability::Operation;
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::domain::{Episode, PlaybackStatus, Subscription};

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
