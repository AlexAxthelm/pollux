use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
pub struct Subscription {
    pub id: String,
    pub feed_url: String,
    pub title: String,
    pub artwork_url: Option<String>,
    pub description: Option<String>,
    pub last_refreshed: Option<i64>,
    pub created_at: i64,
    /// Validators from the last successful fetch, replayed as a conditional GET.
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    /// Why the most recent refresh failed; cleared by the next success.
    pub last_refresh_error: Option<String>,
    /// Unix time before which the host asked us not to refetch (429 `Retry-After`).
    pub retry_after_until: Option<i64>,
}
