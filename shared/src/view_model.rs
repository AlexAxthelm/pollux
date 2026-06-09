use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct ViewModel {
    pub library: LibraryView,
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
