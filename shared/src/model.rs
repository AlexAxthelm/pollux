use crate::domain::Subscription;

#[derive(Default)]
pub struct Model {
    pub subscriptions: Vec<Subscription>,
    pub loading: bool,
    pub fetching_feed: Option<String>,
    pub error: Option<String>,
}
