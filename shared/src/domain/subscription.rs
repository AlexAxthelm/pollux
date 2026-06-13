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
}
