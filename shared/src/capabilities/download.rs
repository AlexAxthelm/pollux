use crux_core::capability::Operation;
use facet::Facet;
use serde::{Deserialize, Serialize};

/// Side-effects the shell performs against episode audio files. Actual network
/// and filesystem I/O lives in the shell; the core only decides *when* to fetch
/// or delete and records the outcome. Downloads are driven one at a time — the
/// serial guard is in the core (see `Model::downloading`), so each operation the
/// shell receives is an independent one-shot.
#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum DownloadOperation {
    /// Fetch `url` and store it locally for `episode_id`.
    Download { episode_id: String, url: String },
    /// Remove a previously downloaded file. Idempotent: deleting a file that is
    /// already gone still succeeds.
    Delete { local_path: String },
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum DownloadResult {
    /// The download finished; the file is at `local_path` (relative to the app's
    /// storage root, resolved to absolute by the shell) and is `size_bytes` long.
    Completed {
        local_path: String,
        size_bytes: u64,
    },
    /// A delete finished (or the file was already absent).
    Deleted,
    Error(String),
}

// Interim download progress is NOT a `DownloadResult`: it is reported by the shell
// as an ordinary `Event::DownloadProgress` (received/total bytes) while the one-shot
// `Download` request is still in flight. That keeps the download a simple one-shot
// (terminal Completed/Error) and keeps progress — which is transient and never
// persisted — out of the storage-backed result type.

impl Operation for DownloadOperation {
    type Output = DownloadResult;
}
