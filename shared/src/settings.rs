//! Settings and how a value is resolved from them.
//!
//! A [`Settings`] holds only the values the user has *overridden* at one level: every field
//! is an `Option`, and `None` means "not set here, use the next level up". Resetting a
//! setting is therefore just setting it back to `None`. The app's own defaults are the
//! constants in [`crate::defaults`], and [`ResolvedSettings`] is what you get once the
//! layers have been walked and every field has a concrete value.
//!
//! The levels are global → subscription → playlist(s) → episode, and an episode's chain is
//! the context it is played from (ADR-0044). That is the caller's business: [`resolve`]
//! only takes the layers that apply, most local first, and the first one that sets a value
//! wins. Adding a level is adding a layer to the list, not a change here.
//!
//! Only the global level exists so far. It is saved as one JSON document per level, written
//! by the core and kept opaque by the shell (ADR-0045).

use crux_core::{render::render, Command};
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::capabilities::storage::{StorageOperation, StorageResult};
use crate::defaults::{RESUME_REWIND_SECS, SKIP_BACKWARD_SECS, SKIP_FORWARD_SECS};
use crate::effect::Effect;
use crate::model::Model;
use crate::Event;

type Cmd = Command<Effect, Event>;

/// The settings overridden at one level. Every field is `None` until the user sets it.
///
/// All levels share this one shape so the resolution is uniform (see `docs/DATA_MODEL.md`).
/// Whether a given setting is *offered* at a level is the UI's concern, not this type's.
///
/// Serialized as JSON holding only the values that are set, so a setting added later needs
/// no migration and an unset one follows the app default (and any change to it). Unknown
/// keys are ignored on load, so a downgrade or a removed setting doesn't break reading.
/// This type never crosses the FFI boundary (bincode can't skip fields); the shell sees the
/// JSON as an opaque string.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    /// Seconds the forward skip button jumps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_forward_secs: Option<u32>,
    /// Seconds the back skip button jumps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_back_secs: Option<u32>,
    /// Seconds playback rewinds when it resumes, so the listener hears what they missed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_rewind_secs: Option<u32>,
}

/// The level a [`Settings`] belongs to. Only the global level exists so far; a subscription
/// or a playlist adds a variant carrying its id, as `EpisodeSource` does.
#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum SettingsScope {
    Global,
}

/// One setting changed at one level: `Some` sets it, `None` resets it to the next level up
/// (or the app default). A setting added later is a variant here and a field on [`Settings`].
#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum SettingChange {
    SkipForwardSecs(Option<u32>),
    SkipBackSecs(Option<u32>),
    ResumeRewindSecs(Option<u32>),
}

impl Settings {
    /// The JSON the shell stores: only the values that are set, so nothing set is `{}`.
    pub(crate) fn to_json(&self) -> String {
        // A struct of optional integers always serializes.
        serde_json::to_string(self).expect("settings always serialize")
    }

    /// Reads stored JSON. Missing keys are unset and unknown keys are ignored.
    pub(crate) fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    fn apply(&mut self, change: SettingChange) {
        match change {
            SettingChange::SkipForwardSecs(v) => self.skip_forward_secs = v,
            SettingChange::SkipBackSecs(v) => self.skip_back_secs = v,
            SettingChange::ResumeRewindSecs(v) => self.resume_rewind_secs = v,
        }
    }
}

/// Every setting with a concrete value: the result of [`resolve`].
///
/// `Default` is the app's own defaults, which is also what an empty list of layers resolves
/// to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedSettings {
    pub skip_forward_secs: u32,
    pub skip_back_secs: u32,
    pub resume_rewind_secs: u32,
}

impl Default for ResolvedSettings {
    fn default() -> Self {
        Self {
            skip_forward_secs: SKIP_FORWARD_SECS,
            skip_back_secs: SKIP_BACKWARD_SECS,
            resume_rewind_secs: RESUME_REWIND_SECS,
        }
    }
}

/// Resolves every setting from `layers`, which must be ordered most local first. For each
/// setting the first layer that sets it wins, and a setting no layer sets takes the app
/// default. Settings resolve independently, so one layer can win some and lose others.
///
/// Cheap enough to call whenever a value is needed, but not from a loop that runs on every
/// position tick: resolve once when playback starts or on the user's action.
pub fn resolve<'a>(layers: impl IntoIterator<Item = &'a Settings> + Clone) -> ResolvedSettings {
    let defaults = ResolvedSettings::default();
    let first = |field: fn(&Settings) -> Option<u32>, default: u32| {
        layers
            .clone()
            .into_iter()
            .find_map(field)
            .unwrap_or(default)
    };
    ResolvedSettings {
        skip_forward_secs: first(|s| s.skip_forward_secs, defaults.skip_forward_secs),
        skip_back_secs: first(|s| s.skip_back_secs, defaults.skip_back_secs),
        resume_rewind_secs: first(|s| s.resume_rewind_secs, defaults.resume_rewind_secs),
    }
}

/// Asks the shell for the saved global settings; the answer is `Event::SettingsLoaded`.
pub(crate) fn load_global() -> Cmd {
    Command::request_from_shell(StorageOperation::LoadSettings {
        scope: SettingsScope::Global,
    })
    .then_send(|r| Event::SettingsLoaded(Box::new(r)))
}

/// The saved global settings arrived at launch. Nothing saved, a failed read or JSON that
/// can't be read all leave the settings at their defaults: the player works either way, and
/// refusing to start over a preference would be worse. The cost is that the next change
/// saves over whatever could not be read.
pub(crate) fn on_loaded(model: &mut Model, result: StorageResult) -> Cmd {
    if let StorageResult::Settings(json) = result {
        if let Ok(settings) = Settings::from_json(&json) {
            model.global_settings = settings;
        }
    }
    render()
}

/// The user set, or reset, one global setting. It takes effect at once, then is saved.
pub(crate) fn set_global(model: &mut Model, change: SettingChange) -> Cmd {
    model.global_settings.apply(change);
    Command::request_from_shell(StorageOperation::SaveSettings {
        scope: SettingsScope::Global,
        json: model.global_settings.to_json(),
    })
    .then_send(|r| Event::SettingsSaved(Box::new(r)))
    .and(render())
}

/// The settings write resolved. A failure leaves the change in effect for this run but
/// unsaved, so it is lost at the next launch; nothing tells the user yet.
pub(crate) fn on_saved(_result: StorageResult) -> Cmd {
    Command::done()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skip_forward(secs: u32) -> Settings {
        Settings {
            skip_forward_secs: Some(secs),
            ..Settings::default()
        }
    }

    #[test]
    fn a_new_settings_overrides_nothing() {
        let s = Settings::default();
        assert_eq!(s.skip_forward_secs, None);
        assert_eq!(s.skip_back_secs, None);
        assert_eq!(s.resume_rewind_secs, None);
    }

    #[test]
    fn no_layers_resolve_to_the_app_defaults() {
        assert_eq!(resolve([]), ResolvedSettings::default());
        assert_eq!(resolve([&Settings::default()]), ResolvedSettings::default());
    }

    #[test]
    fn the_app_defaults_are_the_constants() {
        let d = ResolvedSettings::default();
        assert_eq!(d.skip_forward_secs, SKIP_FORWARD_SECS);
        assert_eq!(d.skip_back_secs, SKIP_BACKWARD_SECS);
        assert_eq!(d.resume_rewind_secs, RESUME_REWIND_SECS);
    }

    #[test]
    fn a_set_value_overrides_the_default() {
        let global = skip_forward(60);
        assert_eq!(resolve([&global]).skip_forward_secs, 60);
    }

    #[test]
    fn zero_is_a_value_not_unset() {
        // A rewind of 0 turns the rewind off; it must not be mistaken for "not set".
        let global = Settings {
            resume_rewind_secs: Some(0),
            ..Settings::default()
        };
        assert_eq!(resolve([&global]).resume_rewind_secs, 0);
    }

    #[test]
    fn the_most_local_layer_wins() {
        let local = skip_forward(20);
        let global = skip_forward(60);
        assert_eq!(resolve([&local, &global]).skip_forward_secs, 20);
    }

    #[test]
    fn an_unset_local_value_falls_through_to_the_next_layer() {
        let local = Settings::default();
        let middle = skip_forward(45);
        let global = skip_forward(60);
        assert_eq!(resolve([&local, &middle, &global]).skip_forward_secs, 45);
    }

    #[test]
    fn settings_resolve_independently() {
        // The local layer wins skip-back, the global layer wins skip-forward, and rewind
        // is set nowhere so it takes the default.
        let local = Settings {
            skip_back_secs: Some(10),
            ..Settings::default()
        };
        let global = Settings {
            skip_forward_secs: Some(60),
            skip_back_secs: Some(45),
            ..Settings::default()
        };
        let resolved = resolve([&local, &global]);
        assert_eq!(resolved.skip_back_secs, 10);
        assert_eq!(resolved.skip_forward_secs, 60);
        assert_eq!(resolved.resume_rewind_secs, RESUME_REWIND_SECS);
    }

    #[test]
    fn resetting_a_value_returns_it_to_the_next_layer() {
        let mut local = skip_forward(20);
        let global = skip_forward(60);
        local.skip_forward_secs = None;
        assert_eq!(resolve([&local, &global]).skip_forward_secs, 60);
    }

    // -- Saving and loading --

    use crux_core::App;

    use crate::Pollux;

    fn send(model: &mut Model, event: Event) -> Vec<Effect> {
        Pollux.update(event, model).effects().collect()
    }

    fn storage_ops(effects: &[Effect]) -> Vec<StorageOperation> {
        effects
            .iter()
            .filter_map(|e| match e {
                Effect::Storage(r) => Some(r.operation.clone()),
                _ => None,
            })
            .collect()
    }

    fn loaded(json: &str) -> Event {
        Event::SettingsLoaded(Box::new(StorageResult::Settings(json.to_string())))
    }

    #[test]
    fn nothing_set_is_an_empty_json_object() {
        assert_eq!(Settings::default().to_json(), "{}");
    }

    #[test]
    fn only_the_values_that_are_set_are_written() {
        let s = Settings {
            skip_forward_secs: Some(60),
            resume_rewind_secs: Some(0),
            ..Settings::default()
        };
        assert_eq!(
            s.to_json(),
            r#"{"skip_forward_secs":60,"resume_rewind_secs":0}"#
        );
    }

    #[test]
    fn json_round_trips() {
        let s = Settings {
            skip_forward_secs: Some(45),
            skip_back_secs: Some(10),
            resume_rewind_secs: Some(5),
        };
        assert_eq!(Settings::from_json(&s.to_json()).unwrap(), s);
    }

    #[test]
    fn missing_keys_are_unset_and_unknown_keys_are_ignored() {
        let s =
            Settings::from_json(r#"{"skip_back_secs":20,"from_a_newer_version":true}"#).unwrap();
        assert_eq!(
            s,
            Settings {
                skip_back_secs: Some(20),
                ..Settings::default()
            }
        );
        assert_eq!(Settings::from_json("{}").unwrap(), Settings::default());
    }

    #[test]
    fn launch_asks_for_the_saved_global_settings() {
        let mut model = Model::default();
        let effects = send(&mut model, Event::Started);
        assert!(storage_ops(&effects).iter().any(|op| matches!(
            op,
            StorageOperation::LoadSettings {
                scope: SettingsScope::Global
            }
        )));
    }

    #[test]
    fn loaded_settings_take_effect() {
        let mut model = Model::default();
        send(&mut model, loaded(r#"{"skip_forward_secs":60}"#));
        assert_eq!(model.global_settings.skip_forward_secs, Some(60));
        assert_eq!(resolve([&model.global_settings]).skip_forward_secs, 60);
    }

    #[test]
    fn nothing_saved_leaves_the_defaults() {
        let mut model = Model::default();
        send(
            &mut model,
            Event::SettingsLoaded(Box::new(StorageResult::NotFound)),
        );
        assert_eq!(model.global_settings, Settings::default());
    }

    #[test]
    fn unreadable_settings_leave_the_defaults() {
        for event in [
            loaded("not json"),
            loaded(r#"{"skip_forward_secs":"sixty"}"#),
            Event::SettingsLoaded(Box::new(StorageResult::Error("db locked".into()))),
        ] {
            let mut model = Model::default();
            send(&mut model, event);
            assert_eq!(model.global_settings, Settings::default());
        }
    }

    #[test]
    fn changing_a_setting_takes_effect_and_saves_only_what_is_overridden() {
        let mut model = Model::default();
        let effects = send(
            &mut model,
            Event::SetGlobalSetting(SettingChange::SkipForwardSecs(Some(60))),
        );
        assert_eq!(model.global_settings.skip_forward_secs, Some(60));
        assert!(matches!(
            storage_ops(&effects).as_slice(),
            [StorageOperation::SaveSettings { scope: SettingsScope::Global, json }]
                if json == r#"{"skip_forward_secs":60}"#
        ));
        assert!(effects.iter().any(|e| matches!(e, Effect::Render(_))));
    }

    #[test]
    fn a_change_keeps_the_other_overrides() {
        let mut model = Model::default();
        send(&mut model, loaded(r#"{"skip_back_secs":10}"#));
        let effects = send(
            &mut model,
            Event::SetGlobalSetting(SettingChange::ResumeRewindSecs(Some(5))),
        );
        let ops = storage_ops(&effects);
        let [StorageOperation::SaveSettings { json, .. }] = ops.as_slice() else {
            panic!("expected one save, got {ops:?}");
        };
        assert_eq!(
            Settings::from_json(json).unwrap(),
            Settings {
                skip_back_secs: Some(10),
                resume_rewind_secs: Some(5),
                ..Settings::default()
            }
        );
    }

    #[test]
    fn resetting_a_setting_removes_it_from_what_is_saved() {
        let mut model = Model::default();
        send(&mut model, loaded(r#"{"skip_forward_secs":60}"#));
        let effects = send(
            &mut model,
            Event::SetGlobalSetting(SettingChange::SkipForwardSecs(None)),
        );
        assert_eq!(model.global_settings, Settings::default());
        assert!(matches!(
            storage_ops(&effects).as_slice(),
            [StorageOperation::SaveSettings { json, .. }] if json == "{}"
        ));
    }

    #[test]
    fn the_settings_write_resolving_does_nothing_more() {
        let mut model = Model::default();
        for result in [StorageResult::Success, StorageResult::Error("full".into())] {
            let effects = send(&mut model, Event::SettingsSaved(Box::new(result)));
            assert!(effects.is_empty());
        }
    }
}
