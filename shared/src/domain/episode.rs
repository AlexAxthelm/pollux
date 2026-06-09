use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[repr(C)]
pub enum PlaybackStatus {
    Unplayed,
    InProgress,
    Played,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[repr(C)]
pub enum DownloadStatus {
    NotDownloaded,
    Queued,
    Downloading,
    Downloaded,
    Failed,
    RemovedFromFeed,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
pub struct Episode {
    pub id: String,
    pub feed_guid: String,
    pub subscription_id: String,
    pub title: String,
    pub description: Option<String>,
    pub pub_date: Option<i64>,
    pub duration_secs: Option<u32>,
    pub enclosure_url: String,
    pub artwork_url: Option<String>,
    pub playback_status: PlaybackStatus,
    pub playback_position_secs: Option<u32>,
    pub download_status: DownloadStatus,
    pub download_progress: Option<u8>,
    pub is_flagged: bool,
    pub file_size_bytes: Option<u64>,
    pub local_path: Option<String>,
}
