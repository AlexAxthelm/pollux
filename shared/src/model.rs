use crate::domain::{Episode, EpisodeSortOrder, Subscription};

#[derive(Default)]
pub struct Model {
    pub subscriptions: Vec<Subscription>,
    pub loading: bool,
    pub error: Option<String>,

    // Subscription details page. Kept separate from the library `loading`/`error`
    // above so opening a feed's episode list never clobbers the Library state.
    pub selected_subscription: Option<Subscription>,
    pub episodes: Vec<Episode>,
    pub episode_sort: EpisodeSortOrder,
    pub detail_loading: bool,
    pub detail_error: Option<String>,
}
