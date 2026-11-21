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
use crate::view_model::{SettingValue, SettingsView};
use crate::Event;

type Cmd = Command<Effect, Event>;

/// The settings overridden at one level. Every field is `None` until the user sets it.
///
/// All levels share this one shape so the resolution is uniform (see `docs/DATA_MODEL.md`).
/// Whether a given setting is *offered* at a level is the UI's concern, not this type's.
///
/// Serialized as JSON holding only the values that are set, so a setting added later needs
/// no migration and an unset one follows the app default (and any change to it). Each key is
/// read on its own: a key that is missing or has a bad value is unset, and the others are
/// kept, so one bad setting doesn't cost the user the rest. Keys this build doesn't know are
/// kept in `unknown` and written back unchanged, so a newer build's settings survive a save
/// by an older one.
/// This type never crosses the FFI boundary (bincode can't skip fields); the shell sees the
/// JSON as an opaque string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// Seconds the forward skip button jumps.
    pub skip_forward_secs: Option<u32>,
    /// Seconds the back skip button jumps.
    pub skip_back_secs: Option<u32>,
    /// Seconds playback rewinds when it resumes, so the listener hears what they missed.
    pub resume_rewind_secs: Option<u32>,
    /// Keys in the saved document that this build has no setting for, kept as read.
    pub(crate) unknown: serde_json::Map<String, serde_json::Value>,
}

/// The keys this build knows. Anything else in a document is kept in `Settings::unknown`.
const KNOWN_KEYS: [&str; 3] = ["skip_forward_secs", "skip_back_secs", "resume_rewind_secs"];

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
    /// The JSON the shell stores: the values that are set, plus any keys this build doesn't
    /// know, so nothing set is `{}`. A known key that is unset is left out.
    pub(crate) fn to_json(&self) -> String {
        let mut map = self.unknown.clone();
        for (key, value) in [
            ("skip_forward_secs", self.skip_forward_secs),
            ("skip_back_secs", self.skip_back_secs),
            ("resume_rewind_secs", self.resume_rewind_secs),
        ] {
            if let Some(value) = value {
                map.insert(key.to_string(), value.into());
            }
        }
        serde_json::Value::Object(map).to_string()
    }

    /// Reads stored JSON, keeping what it can. `None` only when the document isn't a JSON
    /// object at all. Within an object each key is read alone: a known one that is null or not
    /// a whole number that fits is unset, and the others still count. Keys that aren't known
    /// are kept as they are. It doesn't try to repair anything, so a truncated document is
    /// unreadable as a whole.
    pub(crate) fn from_json(json: &str) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_str(json).ok()?;
        let map = value.as_object()?;
        let secs = |key: &str| {
            map.get(key)
                .and_then(serde_json::Value::as_u64)
                .and_then(|n| u32::try_from(n).ok())
        };
        Some(Self {
            skip_forward_secs: secs("skip_forward_secs"),
            skip_back_secs: secs("skip_back_secs"),
            resume_rewind_secs: secs("resume_rewind_secs"),
            unknown: map
                .iter()
                .filter(|(key, _)| !KNOWN_KEYS.contains(&key.as_str()))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        })
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

/// Projects one level's settings for its page. `is_set` is whether the level overrides the
/// setting, `value` is what is in effect there, and `default` is what it would be if reset:
/// for the global level, the app default.
pub(crate) fn settings_view(settings: &Settings) -> SettingsView {
    let defaults = ResolvedSettings::default();
    let value = |set: Option<u32>, default: u32| SettingValue {
        value: set.unwrap_or(default),
        reset_value: default,
        is_set: set.is_some(),
    };
    SettingsView {
        skip_forward_secs: value(settings.skip_forward_secs, defaults.skip_forward_secs),
        skip_back_secs: value(settings.skip_back_secs, defaults.skip_back_secs),
        resume_rewind_secs: value(settings.resume_rewind_secs, defaults.resume_rewind_secs),
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
/// refusing to start over a preference would be worse.
///
/// What was saved is only treated as answered when the shell actually answered. A document
/// that can't be parsed stays where it is until the user changes a setting, which replaces
/// it. A failed read is different: the saved document may be fine, so saving is held and the
/// change stays in memory only, rather than overwriting it (US-0235).
pub(crate) fn on_loaded(model: &mut Model, result: StorageResult) -> Cmd {
    match result {
        StorageResult::Settings(json) => {
            if let Some(settings) = Settings::from_json(&json) {
                model.global_settings = settings;
            }
            model.global_settings_loaded = true;
        }
        StorageResult::NotFound => model.global_settings_loaded = true,
        _ => {}
    }
    render()
}

/// The user set, or reset, one global setting. It takes effect at once, then is saved, unless
/// the saved settings haven't been read successfully yet, in which case it is not saved.
pub(crate) fn set_global(model: &mut Model, change: SettingChange) -> Cmd {
    model.global_settings.apply(change);
    if !model.global_settings_loaded {
        return render();
    }
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

    fn nothing_saved() -> Event {
        Event::SettingsLoaded(Box::new(StorageResult::NotFound))
    }

    fn read_failed() -> Event {
        Event::SettingsLoaded(Box::new(StorageResult::Error("db locked".into())))
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
            r#"{"resume_rewind_secs":0,"skip_forward_secs":60}"#
        );
    }

    #[test]
    fn json_round_trips() {
        let s = Settings {
            skip_forward_secs: Some(45),
            skip_back_secs: Some(10),
            resume_rewind_secs: Some(5),
            ..Settings::default()
        };
        assert_eq!(Settings::from_json(&s.to_json()).unwrap(), s);
    }

    #[test]
    fn one_bad_value_does_not_cost_the_others() {
        let s = Settings::from_json(
            r#"{"skip_forward_secs":"sixty","skip_back_secs":20,"resume_rewind_secs":-3}"#,
        )
        .unwrap();
        assert_eq!(
            s,
            Settings {
                skip_back_secs: Some(20),
                ..Settings::default()
            }
        );
        for bad in ["null", "1.5", "99999999999", "true", "[]"] {
            let doc = format!(r#"{{"skip_forward_secs":{bad},"skip_back_secs":20}}"#);
            let s = Settings::from_json(&doc).unwrap();
            assert_eq!(s.skip_forward_secs, None, "{bad}");
            assert_eq!(s.skip_back_secs, Some(20), "{bad}");
        }
    }

    #[test]
    fn a_document_that_is_not_an_object_is_unreadable() {
        for bad in [
            "not json",
            r#"{"skip_back_secs":20"#,
            "[1,2]",
            "60",
            "null",
            "",
        ] {
            assert_eq!(Settings::from_json(bad), None, "{bad}");
        }
    }

    #[test]
    fn missing_keys_are_unset() {
        let s = Settings::from_json(r#"{"skip_back_secs":20}"#).unwrap();
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
    fn unknown_keys_are_kept_when_saved_again() {
        let mut s = Settings::from_json(
            r#"{"skip_back_secs":20,"from_a_newer_version":{"on":true},"another":3}"#,
        )
        .unwrap();
        assert_eq!(s.skip_back_secs, Some(20));
        s.apply(SettingChange::SkipForwardSecs(Some(60)));
        let saved: serde_json::Value = serde_json::from_str(&s.to_json()).unwrap();
        assert_eq!(
            saved,
            serde_json::json!({
                "skip_forward_secs": 60,
                "skip_back_secs": 20,
                "from_a_newer_version": {"on": true},
                "another": 3,
            })
        );
    }

    #[test]
    fn resetting_a_known_key_removes_it_but_keeps_the_unknown_ones() {
        let mut s = Settings::from_json(r#"{"skip_back_secs":20,"other":1}"#).unwrap();
        s.apply(SettingChange::SkipBackSecs(None));
        assert_eq!(s.to_json(), r#"{"other":1}"#);
    }

    #[test]
    fn a_bad_known_value_is_dropped_when_saved_but_unknown_keys_stay() {
        let s = Settings::from_json(r#"{"skip_back_secs":"x","other":1}"#).unwrap();
        assert_eq!(s.to_json(), r#"{"other":1}"#);
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
            loaded("[1,2]"),
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
        send(&mut model, nothing_saved());
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
    fn a_change_is_not_saved_before_the_load_answers() {
        let mut model = Model::default();
        let effects = send(
            &mut model,
            Event::SetGlobalSetting(SettingChange::SkipForwardSecs(Some(60))),
        );
        assert_eq!(model.global_settings.skip_forward_secs, Some(60));
        assert!(storage_ops(&effects).is_empty());
    }

    #[test]
    fn a_failed_read_holds_saving_so_the_saved_settings_are_not_overwritten() {
        let mut model = Model::default();
        send(&mut model, read_failed());
        let effects = send(
            &mut model,
            Event::SetGlobalSetting(SettingChange::SkipForwardSecs(Some(60))),
        );
        assert_eq!(model.global_settings.skip_forward_secs, Some(60));
        assert!(storage_ops(&effects).is_empty());
        assert!(effects.iter().any(|e| matches!(e, Effect::Render(_))));
    }

    #[test]
    fn an_unparseable_document_is_replaced_by_the_next_change() {
        let mut model = Model::default();
        send(&mut model, loaded("not json"));
        let effects = send(
            &mut model,
            Event::SetGlobalSetting(SettingChange::SkipForwardSecs(Some(60))),
        );
        assert!(matches!(
            storage_ops(&effects).as_slice(),
            [StorageOperation::SaveSettings { json, .. }]
                if json == r#"{"skip_forward_secs":60}"#
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

    // -- The settings page --

    #[test]
    fn nothing_set_shows_every_setting_at_its_app_default() {
        let view = settings_view(&Settings::default());
        let defaults = ResolvedSettings::default();
        for (shown, default) in [
            (view.skip_forward_secs, defaults.skip_forward_secs),
            (view.skip_back_secs, defaults.skip_back_secs),
            (view.resume_rewind_secs, defaults.resume_rewind_secs),
        ] {
            assert_eq!(
                shown,
                SettingValue {
                    value: default,
                    reset_value: default,
                    is_set: false
                }
            );
        }
    }

    #[test]
    fn a_set_value_shows_as_set_and_remembers_what_a_reset_would_give() {
        let view = settings_view(&skip_forward(60));
        assert_eq!(
            view.skip_forward_secs,
            SettingValue {
                value: 60,
                reset_value: ResolvedSettings::default().skip_forward_secs,
                is_set: true
            }
        );
        assert!(!view.skip_back_secs.is_set);
    }

    #[test]
    fn a_value_set_to_the_default_still_counts_as_set() {
        let default = ResolvedSettings::default().skip_forward_secs;
        let view = settings_view(&skip_forward(default));
        assert!(view.skip_forward_secs.is_set);
    }

    #[test]
    fn the_view_model_carries_the_settings_and_follows_changes() {
        let mut model = Model::default();
        assert!(!Pollux.view(&model).settings.skip_back_secs.is_set);
        send(&mut model, nothing_saved());
        send(
            &mut model,
            Event::SetGlobalSetting(SettingChange::SkipBackSecs(Some(10))),
        );
        let shown = Pollux.view(&model).settings.skip_back_secs;
        assert_eq!((shown.value, shown.is_set), (10, true));
        send(
            &mut model,
            Event::SetGlobalSetting(SettingChange::SkipBackSecs(None)),
        );
        assert!(!Pollux.view(&model).settings.skip_back_secs.is_set);
    }

    #[test]
    fn the_default_view_is_every_setting_at_its_default() {
        let view = SettingsView::default();
        assert_eq!(view.skip_forward_secs.value, SKIP_FORWARD_SECS);
        assert!(!view.skip_forward_secs.is_set);
    }
}
