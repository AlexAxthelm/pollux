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
//! Only the global level exists so far, and nothing reads it yet.

use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::defaults::{RESUME_REWIND_SECS, SKIP_BACKWARD_SECS, SKIP_FORWARD_SECS};

/// The settings overridden at one level. Every field is `None` until the user sets it.
///
/// All levels share this one shape so the resolution is uniform (see `docs/DATA_MODEL.md`).
/// Whether a given setting is *offered* at a level is the UI's concern, not this type's.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// Seconds the forward skip button jumps.
    pub skip_forward_secs: Option<u32>,
    /// Seconds the back skip button jumps.
    pub skip_back_secs: Option<u32>,
    /// Seconds playback rewinds when it resumes, so the listener hears what they missed.
    pub resume_rewind_secs: Option<u32>,
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
}
