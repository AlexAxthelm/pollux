use crate::domain::Subscription;

#[derive(Default)]
pub struct Model {
    pub subscriptions: Vec<Subscription>,
    pub loading: bool,
    pub error: Option<String>,
}
