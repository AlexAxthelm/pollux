use crate::domain::{EpisodeSortOrder, Subscription};
use crate::view_model::EpisodeSummary;

#[derive(Default)]
pub struct Model {
    pub subscriptions: Vec<Subscription>,
    pub loading: bool,
    pub error: Option<String>,

    // Subscription details page. Kept separate from the library `loading`/`error`
    // above so opening a feed's episode list never clobbers the Library state.
    pub selected_subscription: Option<Subscription>,
    // Episodes are projected to display summaries once, when they load, rather
    // than on every render: the HTML-stripping in that projection is not free, and
    // `view()` runs for unrelated events too. Ordering still happens in `view()`.
    pub episode_summaries: Vec<EpisodeSummary>,
    pub episode_sort: EpisodeSortOrder,
    pub detail_loading: bool,
    pub detail_error: Option<String>,
}
