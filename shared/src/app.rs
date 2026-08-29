use crux_core::{render::render, App, Command};
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::capabilities::http::{HttpOperation, HttpResult};
use crate::capabilities::storage::{StorageOperation, StorageResult};
use crate::domain::EpisodeSortOrder;
use crate::effect::Effect;
use crate::feed_parser::parse_feed;
use crate::html::strip_html;
use crate::model::Model;
use crate::view_model::{
    EpisodeSummary, LibraryView, SubscriptionDetailView, SubscriptionSummary, ViewModel,
};

#[derive(Default)]
pub struct Pollux;

impl App for Pollux {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
        match event {
            Event::Started => {
                model.loading = true;
                Command::request_from_shell(StorageOperation::ListSubscriptions)
                    .then_send(|r| Event::SubscriptionsLoaded(Box::new(r)))
                    .and(render())
            }
            Event::SubscriptionsLoaded(result) => {
                model.loading = false;
                match *result {
                    StorageResult::Subscriptions(rows) => {
                        model.subscriptions = rows;
                        model.error = None;
                    }
                    StorageResult::Error(e) => model.error = Some(e),
                    unexpected => {
                        model.error = Some(format!("unexpected storage result: {unexpected:?}"))
                    }
                }
                render()
            }
            Event::FetchFeed(url) => {
                model.loading = true;
                model.error = None;
                Command::request_from_shell(HttpOperation::FetchFeed { url: url.clone() })
                    .then_send(move |r| Event::FeedFetched {
                        url,
                        result: Box::new(r),
                    })
                    .and(render())
            }
            Event::FeedFetched { url, result } => match *result {
                HttpResult::Error(e) => {
                    model.loading = false;
                    model.error = Some(e);
                    render()
                }
                HttpResult::Response { status: 200, body } => match parse_feed(&url, body) {
                    Ok((subscription, episodes)) => {
                        Command::request_from_shell(StorageOperation::UpsertFeedWithEpisodes {
                            subscription,
                            episodes,
                        })
                        .then_send(|r| Event::FeedSaved(Box::new(r)))
                    }
                    Err(e) => {
                        model.loading = false;
                        model.error = Some(e);
                        render()
                    }
                },
                HttpResult::Response { status, .. } => {
                    model.loading = false;
                    model.error = Some(format!("feed fetch failed: HTTP {status}"));
                    render()
                }
            },
            Event::FeedSaved(result) => {
                model.loading = false;
                match *result {
                    StorageResult::Subscription(sub) => {
                        if let Some(pos) = model.subscriptions.iter().position(|s| s.id == sub.id) {
                            model.subscriptions[pos] = sub;
                        } else {
                            model.subscriptions.push(sub);
                        }
                        model.error = None;
                    }
                    StorageResult::Error(e) => model.error = Some(e),
                    unexpected => {
                        model.error = Some(format!("unexpected storage result: {unexpected:?}"))
                    }
                }
                render()
            }
            Event::SelectSubscription(id) => {
                // Remember the chosen feed (for the details header) from the
                // already-loaded library, so the page needs no extra fetch for it.
                model.selected_subscription =
                    model.subscriptions.iter().find(|s| s.id == id).cloned();
                model.episodes.clear();
                model.detail_loading = true;
                model.detail_error = None;
                Command::request_from_shell(StorageOperation::ListEpisodesBySubscription {
                    subscription_id: id,
                })
                .then_send(|r| Event::EpisodesLoaded(Box::new(r)))
                .and(render())
            }
            Event::EpisodesLoaded(result) => {
                model.detail_loading = false;
                match *result {
                    StorageResult::Episodes(rows) => {
                        model.episodes = rows;
                        model.detail_error = None;
                    }
                    StorageResult::Error(e) => model.detail_error = Some(e),
                    unexpected => {
                        model.detail_error =
                            Some(format!("unexpected storage result: {unexpected:?}"))
                    }
                }
                render()
            }
            Event::SetEpisodeSort(order) => {
                // The sort is applied in view(), so this only records the choice
                // and re-renders — no storage round-trip needed.
                model.episode_sort = order;
                render()
            }
        }
    }

    fn view(&self, model: &Model) -> ViewModel {
        // Display order is owned here rather than at each mutation site, so the
        // core does not depend on the shell returning rows in any given order.
        let mut subscriptions: Vec<SubscriptionSummary> = model
            .subscriptions
            .iter()
            .map(|s| SubscriptionSummary {
                id: s.id.clone(),
                title: s.title.clone(),
                artwork_url: s.artwork_url.clone(),
            })
            .collect();
        subscriptions.sort_by_cached_key(|s| s.title.to_lowercase());

        ViewModel {
            library: LibraryView {
                subscriptions,
                loading: model.loading,
                error: model.error.clone(),
            },
            subscription_detail: build_subscription_detail(model),
        }
    }
}

/// Builds the details-page view from the selected subscription and its episodes.
/// Display order is owned here (like `subscriptions` above) so the core stays the
/// single source of truth for sort order, independent of how the shell's SQL
/// returned the rows.
fn build_subscription_detail(model: &Model) -> SubscriptionDetailView {
    let mut episodes: Vec<EpisodeSummary> = model
        .episodes
        .iter()
        .map(|e| EpisodeSummary {
            id: e.id.clone(),
            title: e.title.clone(),
            description: e.description.clone(),
            description_text: e
                .description
                .as_deref()
                .map(strip_html)
                .filter(|s| !s.is_empty()),
            pub_date: e.pub_date,
            duration_secs: e.duration_secs,
            artwork_url: e.artwork_url.clone(),
            playback_status: e.playback_status.clone(),
            playback_position_secs: e.playback_position_secs,
            download_status: e.download_status.clone(),
            download_progress: e.download_progress,
        })
        .collect();
    sort_episodes(&mut episodes, model.episode_sort);

    let (subscription_id, title, artwork_url) = match &model.selected_subscription {
        Some(s) => (Some(s.id.clone()), s.title.clone(), s.artwork_url.clone()),
        None => (None, String::new(), None),
    };

    SubscriptionDetailView {
        subscription_id,
        title,
        artwork_url,
        episodes,
        sort_order: model.episode_sort,
        loading: model.detail_loading,
        error: model.detail_error.clone(),
    }
}

/// Sorts episodes in place for display. For both date orders, episodes with no
/// `pub_date` sort last so undated items never crowd out the meaningful ordering.
fn sort_episodes(episodes: &mut [EpisodeSummary], order: EpisodeSortOrder) {
    match order {
        EpisodeSortOrder::PubDateDesc => {
            // Missing dates last: key on (is_missing, negated date) ascending.
            episodes.sort_by_key(|e| (e.pub_date.is_none(), e.pub_date.map(|d| -d)));
        }
        EpisodeSortOrder::PubDateAsc => {
            episodes.sort_by_key(|e| (e.pub_date.is_none(), e.pub_date));
        }
        EpisodeSortOrder::TitleAsc => {
            episodes.sort_by_cached_key(|e| e.title.to_lowercase());
        }
    }
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum Event {
    Started,
    SubscriptionsLoaded(Box<StorageResult>),
    FetchFeed(String),
    FeedFetched {
        url: String,
        result: Box<HttpResult>,
    },
    FeedSaved(Box<StorageResult>),
    SelectSubscription(String),
    EpisodesLoaded(Box<StorageResult>),
    SetEpisodeSort(EpisodeSortOrder),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DownloadStatus, Episode, PlaybackStatus, Subscription};
    use crate::effect::Effect;

    fn make_subscription(id: &str, title: &str) -> Subscription {
        Subscription {
            id: id.to_string(),
            feed_url: format!("https://example.com/{id}.rss"),
            title: title.to_string(),
            artwork_url: None,
            description: None,
            last_refreshed: None,
            created_at: 0,
        }
    }

    fn make_episode(id: &str, title: &str, pub_date: Option<i64>) -> Episode {
        Episode {
            id: id.to_string(),
            feed_guid: format!("{id}-guid"),
            subscription_id: "sub-id".to_string(),
            title: title.to_string(),
            description: None,
            pub_date,
            duration_secs: None,
            enclosure_url: format!("https://example.com/{id}.mp3"),
            artwork_url: None,
            playback_status: PlaybackStatus::Unplayed,
            playback_position_secs: None,
            download_status: DownloadStatus::NotDownloaded,
            download_progress: None,
            is_flagged: false,
            file_size_bytes: None,
            local_path: None,
        }
    }

    fn detail_titles(app: &Pollux, model: &Model) -> Vec<String> {
        app.view(model)
            .subscription_detail
            .episodes
            .into_iter()
            .map(|e| e.title)
            .collect()
    }

    fn view_titles(app: &Pollux, model: &Model) -> Vec<String> {
        app.view(model)
            .library
            .subscriptions
            .into_iter()
            .map(|s| s.title)
            .collect()
    }

    const MINIMAL_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
<channel>
  <title>Test Podcast</title>
  <description>A test feed</description>
  <link>https://example.com</link>
  <item>
    <title>Episode 1</title>
    <guid>episode-1-guid</guid>
    <pubDate>Mon, 01 Jan 2024 00:00:00 +0000</pubDate>
    <itunes:duration>3600</itunes:duration>
    <enclosure url="https://example.com/ep1.mp3" type="audio/mpeg" length="12345678"/>
  </item>
</channel>
</rss>"#;

    #[test]
    fn init_sets_loading_and_issues_storage_request() {
        let app = Pollux;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Started, &mut model);

        assert!(model.loading);

        let effects: Vec<Effect> = cmd.effects().collect();
        assert_eq!(effects.len(), 2);

        let has_storage = effects
            .iter()
            .any(|e| matches!(e, Effect::Storage(r) if matches!(r.operation, StorageOperation::ListSubscriptions)));
        assert!(has_storage, "expected a ListSubscriptions storage effect");

        let has_render = effects.iter().any(|e| matches!(e, Effect::Render(_)));
        assert!(has_render, "expected a render effect");
    }

    #[test]
    fn subscriptions_loaded_updates_view() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let subs = vec![
            make_subscription("id-1", "Podcast One"),
            make_subscription("id-2", "Podcast Two"),
        ];
        let mut cmd = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Subscriptions(subs))),
            &mut model,
        );

        assert!(!model.loading);
        assert!(
            model.error.is_none(),
            "successful load should clear previous error"
        );
        cmd.expect_one_effect().expect_render();

        let view = app.view(&model);
        assert_eq!(view.library.subscriptions.len(), 2);
        assert_eq!(view.library.subscriptions[0].id, "id-1");
        assert_eq!(view.library.subscriptions[0].title, "Podcast One");
        assert_eq!(view.library.subscriptions[1].id, "id-2");
    }

    #[test]
    fn empty_subscriptions_loaded_shows_empty_state() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let mut cmd = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Subscriptions(vec![]))),
            &mut model,
        );

        assert!(!model.loading);
        cmd.expect_one_effect().expect_render();

        let view = app.view(&model);
        assert!(view.library.subscriptions.is_empty());
        assert!(!view.library.loading);
        assert!(view.library.error.is_none());
    }

    #[test]
    fn storage_error_sets_error_state() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let _ = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Error(
                "db unavailable".to_string(),
            ))),
            &mut model,
        );

        assert!(!model.loading);
        assert_eq!(model.error.as_deref(), Some("db unavailable"));
    }

    #[test]
    fn successful_load_after_error_clears_error() {
        let app = Pollux;
        let mut model = Model::default();

        let _ = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Error(
                "transient failure".to_string(),
            ))),
            &mut model,
        );
        assert!(model.error.is_some());

        let _ = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Subscriptions(vec![
                make_subscription("id-1", "Recovered"),
            ]))),
            &mut model,
        );

        assert!(
            model.error.is_none(),
            "error should be cleared after successful load"
        );
        assert_eq!(model.subscriptions.len(), 1);
    }

    #[test]
    fn unexpected_storage_result_surfaces_as_error() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let _ = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::NotFound)),
            &mut model,
        );

        assert!(!model.loading);
        assert!(model.error.is_some(), "unexpected result should set error");
        assert!(model.subscriptions.is_empty());
    }

    #[test]
    fn fetch_feed_sets_loading_and_emits_http_effect() {
        let app = Pollux;
        let mut model = Model::default();

        let mut cmd = app.update(
            Event::FetchFeed("https://example.com/feed.rss".to_string()),
            &mut model,
        );

        assert!(model.loading);

        let effects: Vec<Effect> = cmd.effects().collect();
        assert_eq!(effects.len(), 2);

        let has_http = effects.iter().any(|e| matches!(
            e,
            Effect::Http(r) if matches!(&r.operation, HttpOperation::FetchFeed { url } if url == "https://example.com/feed.rss")
        ));
        assert!(has_http, "expected a FetchFeed http effect");

        let has_render = effects.iter().any(|e| matches!(e, Effect::Render(_)));
        assert!(has_render, "expected a render effect");
    }

    #[test]
    fn feed_fetched_http_error_sets_error_clears_loading() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let mut cmd = app.update(
            Event::FeedFetched {
                url: "https://example.com/feed.rss".to_string(),
                result: Box::new(HttpResult::Error("connection refused".to_string())),
            },
            &mut model,
        );

        assert!(!model.loading);
        assert_eq!(model.error.as_deref(), Some("connection refused"));
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn feed_fetched_non_200_sets_error() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let mut cmd = app.update(
            Event::FeedFetched {
                url: "https://example.com/feed.rss".to_string(),
                result: Box::new(HttpResult::Response {
                    status: 404,
                    body: vec![],
                }),
            },
            &mut model,
        );

        assert!(!model.loading);
        assert!(model.error.is_some());
        assert!(model.error.as_deref().is_some_and(|e| e.contains("404")));
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn feed_fetched_valid_rss_emits_upsert_storage_effect() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let url = "https://example.com/feed.rss".to_string();
        let mut cmd = app.update(
            Event::FeedFetched {
                url: url.clone(),
                result: Box::new(HttpResult::Response {
                    status: 200,
                    body: MINIMAL_RSS.as_bytes().to_vec(),
                }),
            },
            &mut model,
        );

        // Should not have cleared loading yet — waiting for storage
        assert!(model.error.is_none(), "valid RSS should not set error");

        let effects: Vec<Effect> = cmd.effects().collect();
        assert_eq!(effects.len(), 1, "expected exactly one storage effect");

        let has_upsert = effects.iter().any(|e| matches!(
            e,
            Effect::Storage(r) if matches!(&r.operation, StorageOperation::UpsertFeedWithEpisodes { subscription, episodes }
                if subscription.feed_url == url && !episodes.is_empty())
        ));
        assert!(has_upsert, "expected UpsertFeedWithEpisodes storage effect");
    }

    #[test]
    fn feed_fetched_invalid_xml_sets_error() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let mut cmd = app.update(
            Event::FeedFetched {
                url: "https://example.com/feed.rss".to_string(),
                result: Box::new(HttpResult::Response {
                    status: 200,
                    body: b"this is not xml".to_vec(),
                }),
            },
            &mut model,
        );

        assert!(!model.loading);
        assert!(model.error.is_some(), "invalid XML should set error");
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn feed_saved_adds_new_subscription_to_model() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let sub = make_subscription("new-id", "New Podcast");
        let mut cmd = app.update(
            Event::FeedSaved(Box::new(StorageResult::Subscription(sub))),
            &mut model,
        );

        assert!(!model.loading);
        assert!(model.error.is_none());
        assert_eq!(model.subscriptions.len(), 1);
        assert_eq!(model.subscriptions[0].title, "New Podcast");
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn feed_saved_updates_existing_subscription_in_model() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![make_subscription("sub-id", "Old Title")];

        let updated = Subscription {
            title: "New Title".to_string(),
            ..make_subscription("sub-id", "New Title")
        };
        let _ = app.update(
            Event::FeedSaved(Box::new(StorageResult::Subscription(updated))),
            &mut model,
        );

        assert_eq!(model.subscriptions.len(), 1);
        assert_eq!(model.subscriptions[0].title, "New Title");
    }

    #[test]
    fn feed_saved_resorts_when_refresh_changes_title() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![
            make_subscription("a-id", "Alpha Podcast"),
            make_subscription("z-id", "Zebra Podcast"),
        ];

        // A refresh renames the first feed so it now sorts last.
        let renamed = make_subscription("a-id", "Zulu Podcast");
        let _ = app.update(
            Event::FeedSaved(Box::new(StorageResult::Subscription(renamed))),
            &mut model,
        );

        assert_eq!(model.subscriptions.len(), 2, "rename should not add a row");
        assert_eq!(
            view_titles(&app, &model),
            vec!["Zebra Podcast", "Zulu Podcast"],
            "view should stay alphabetical after an in-place update"
        );
    }

    #[test]
    fn view_sorts_unordered_subscriptions() {
        let app = Pollux;
        let mut model = Model::default();
        // The shell is not required to return rows in any particular order.
        model.subscriptions = vec![
            make_subscription("c-id", "charlie"),
            make_subscription("a-id", "Alpha"),
            make_subscription("b-id", "Bravo"),
        ];

        assert_eq!(
            view_titles(&app, &model),
            vec!["Alpha", "Bravo", "charlie"],
            "view sorts case-insensitively regardless of model order"
        );
    }

    #[test]
    fn feed_saved_sorted_alphabetically() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![make_subscription("b-id", "Zebra Podcast")];

        let sub = make_subscription("a-id", "Alpha Podcast");
        let _ = app.update(
            Event::FeedSaved(Box::new(StorageResult::Subscription(sub))),
            &mut model,
        );

        assert_eq!(model.subscriptions.len(), 2);
        assert_eq!(
            view_titles(&app, &model),
            vec!["Alpha Podcast", "Zebra Podcast"]
        );
    }

    #[test]
    fn select_subscription_sets_header_and_issues_storage_request() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![make_subscription("sub-id", "My Feed")];

        let mut cmd = app.update(Event::SelectSubscription("sub-id".to_string()), &mut model);

        assert!(model.detail_loading);
        assert!(model.detail_error.is_none());

        // The header comes from the already-loaded library, not a second fetch.
        let view = app.view(&model);
        assert_eq!(
            view.subscription_detail.subscription_id.as_deref(),
            Some("sub-id")
        );
        assert_eq!(view.subscription_detail.title, "My Feed");

        let effects: Vec<Effect> = cmd.effects().collect();
        let has_list = effects.iter().any(|e| {
            matches!(
                e,
                Effect::Storage(r) if matches!(&r.operation,
                    StorageOperation::ListEpisodesBySubscription { subscription_id }
                        if subscription_id == "sub-id")
            )
        });
        assert!(
            has_list,
            "expected a ListEpisodesBySubscription storage effect"
        );
        let has_render = effects.iter().any(|e| matches!(e, Effect::Render(_)));
        assert!(has_render, "expected a render effect");
    }

    #[test]
    fn episodes_loaded_populates_detail_view() {
        let app = Pollux;
        let mut model = Model::default();
        model.detail_loading = true;

        let episodes = vec![
            make_episode("e1", "Episode One", Some(1_000)),
            make_episode("e2", "Episode Two", Some(2_000)),
        ];
        let mut cmd = app.update(
            Event::EpisodesLoaded(Box::new(StorageResult::Episodes(episodes))),
            &mut model,
        );

        assert!(!model.detail_loading);
        assert!(model.detail_error.is_none());
        cmd.expect_one_effect().expect_render();

        let view = app.view(&model);
        assert_eq!(view.subscription_detail.episodes.len(), 2);
    }

    #[test]
    fn episodes_loaded_defaults_to_newest_first() {
        let app = Pollux;
        let mut model = Model::default();
        model.episodes = vec![
            make_episode("old", "Old", Some(1_000)),
            make_episode("new", "New", Some(3_000)),
            make_episode("mid", "Mid", Some(2_000)),
        ];

        assert_eq!(detail_titles(&app, &model), vec!["New", "Mid", "Old"]);
    }

    #[test]
    fn set_episode_sort_reorders_the_view() {
        let app = Pollux;
        let mut model = Model::default();
        model.episodes = vec![
            make_episode("b", "Bravo", Some(3_000)),
            make_episode("a", "Alpha", Some(1_000)),
            make_episode("c", "Charlie", Some(2_000)),
        ];

        let _ = app.update(
            Event::SetEpisodeSort(EpisodeSortOrder::PubDateAsc),
            &mut model,
        );
        assert_eq!(model.episode_sort, EpisodeSortOrder::PubDateAsc);
        assert_eq!(
            detail_titles(&app, &model),
            vec!["Alpha", "Charlie", "Bravo"]
        );

        let _ = app.update(
            Event::SetEpisodeSort(EpisodeSortOrder::TitleAsc),
            &mut model,
        );
        assert_eq!(
            detail_titles(&app, &model),
            vec!["Alpha", "Bravo", "Charlie"]
        );

        let _ = app.update(
            Event::SetEpisodeSort(EpisodeSortOrder::PubDateDesc),
            &mut model,
        );
        assert_eq!(
            detail_titles(&app, &model),
            vec!["Bravo", "Charlie", "Alpha"]
        );
    }

    #[test]
    fn episodes_without_pub_date_sort_last_in_both_date_orders() {
        let app = Pollux;
        let mut model = Model::default();
        model.episodes = vec![
            make_episode("dated-old", "Dated Old", Some(1_000)),
            make_episode("undated", "Undated", None),
            make_episode("dated-new", "Dated New", Some(2_000)),
        ];

        model.episode_sort = EpisodeSortOrder::PubDateDesc;
        assert_eq!(
            detail_titles(&app, &model),
            vec!["Dated New", "Dated Old", "Undated"],
            "undated sorts last, newest first"
        );

        model.episode_sort = EpisodeSortOrder::PubDateAsc;
        assert_eq!(
            detail_titles(&app, &model),
            vec!["Dated Old", "Dated New", "Undated"],
            "undated sorts last, oldest first"
        );
    }

    #[test]
    fn episodes_loaded_storage_error_sets_detail_error() {
        let app = Pollux;
        let mut model = Model::default();
        model.detail_loading = true;

        let _ = app.update(
            Event::EpisodesLoaded(Box::new(StorageResult::Error("db gone".to_string()))),
            &mut model,
        );

        assert!(!model.detail_loading);
        assert_eq!(model.detail_error.as_deref(), Some("db gone"));
    }

    #[test]
    fn selecting_a_subscription_does_not_disturb_library_state() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![make_subscription("sub-id", "My Feed")];
        model.loading = false;
        model.error = Some("library error".to_string());

        let _ = app.update(Event::SelectSubscription("sub-id".to_string()), &mut model);

        // Detail load is in flight, but the library's own flags are untouched.
        assert!(!model.loading);
        assert_eq!(model.error.as_deref(), Some("library error"));
        assert!(model.detail_loading);
    }

    #[test]
    fn episode_summary_exposes_raw_and_stripped_description() {
        let app = Pollux;
        let mut model = Model::default();
        let mut episode = make_episode("e1", "Ep", Some(1_000));
        episode.description = Some("<p>Hello <b>world</b></p>".to_string());
        model.episodes = vec![episode];

        let summary = &app.view(&model).subscription_detail.episodes[0];
        // Raw HTML is preserved for the detail page's rich rendering...
        assert_eq!(
            summary.description.as_deref(),
            Some("<p>Hello <b>world</b></p>")
        );
        // ...and a plain-text version is provided for the row snippet.
        assert_eq!(summary.description_text.as_deref(), Some("Hello world"));
    }

    #[test]
    fn episode_summary_description_text_absent_when_no_description() {
        let app = Pollux;
        let mut model = Model::default();
        // make_episode leaves description as None.
        model.episodes = vec![make_episode("e1", "Ep", Some(1_000))];

        let summary = &app.view(&model).subscription_detail.episodes[0];
        assert!(summary.description.is_none());
        assert!(summary.description_text.is_none());
    }
}
