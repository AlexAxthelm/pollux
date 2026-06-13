use crux_core::{render::render, App, Command};
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::capabilities::storage::{StorageOperation, StorageResult};
use crate::effect::Effect;
use crate::model::Model;
use crate::view_model::{LibraryView, SubscriptionSummary, ViewModel};

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
        }
    }

    fn view(&self, model: &Model) -> ViewModel {
        ViewModel {
            library: LibraryView {
                subscriptions: model
                    .subscriptions
                    .iter()
                    .map(|s| SubscriptionSummary {
                        id: s.id.clone(),
                        title: s.title.clone(),
                        artwork_url: s.artwork_url.clone(),
                    })
                    .collect(),
                loading: model.loading,
                error: model.error.clone(),
            },
        }
    }
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum Event {
    Started,
    SubscriptionsLoaded(Box<StorageResult>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Subscription;
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
}
