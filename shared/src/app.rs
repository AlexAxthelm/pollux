use crux_core::{render::render, App, Command};
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::capabilities::storage::{StorageOperation, StorageResult};
use crate::effect::Effect;
use crate::model::Model;
use crate::view_model::{SubscriptionSummary, ViewModel};

#[derive(Default)]
pub struct Pollux;

impl App for Pollux {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
        match event {
            Event::Init => {
                model.loading = true;
                Command::request_from_shell(StorageOperation::ListSubscriptions)
                    .then_send(Event::SubscriptionsLoaded)
                    .and(render())
            }
            Event::SubscriptionsLoaded(result) => {
                model.loading = false;
                match result {
                    StorageResult::Subscriptions(rows) => model.subscriptions = rows,
                    StorageResult::Error(e) => model.error = Some(e),
                    _ => {}
                }
                render()
            }
        }
    }

    fn view(&self, model: &Model) -> ViewModel {
        ViewModel {
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
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum Event {
    Init,
    SubscriptionsLoaded(StorageResult),
}
