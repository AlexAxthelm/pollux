//! Theme (colorscheme) infrastructure.
//!
//! Colors are defined with the base16 nomenclature (base00–base0F); see
//! <https://github.com/chriskempson/base16>. The core is the single source of
//! truth for palette *data* — the built-in palettes below today, user-defined
//! ones (persisted via storage) later — while the shell only resolves these
//! tokens into platform colors and injects them into the view tree.
//!
//! The active selection (`ThemeId` + `ThemeMode`) lives in the `Model` and is
//! projected into the `ViewModel` as a [`ThemeView`]. There is no UI to change
//! it yet; the Settings appearance section (see `docs/features/theme.md`) will
//! drive it through [`crate::Event::SetTheme`]. Until then the default is
//! `System` / `FollowSystem`, which reproduces the OS's native appearance.

use facet::Facet;
use serde::{Deserialize, Serialize};

/// Whether a theme is pinned to its light or dark variant, or follows the OS.
///
/// A theme with only one variant (e.g. Nord, dark-only) ignores this — the shell
/// uses `has_dark_variant` to know when the choice is meaningful.
#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub enum ThemeMode {
    /// Follow the OS light/dark setting (the default).
    #[default]
    FollowSystem,
    /// Always use the light variant.
    Light,
    /// Always use the dark variant.
    Dark,
}

/// A built-in theme. `Custom` (user-edited hex, persisted) is a future addition;
/// adding a palette is just a new variant plus its data in [`theme_view`].
#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C)]
pub enum ThemeId {
    /// Defer entirely to the OS semantic colors — the app's appearance today.
    #[default]
    System,
    /// Solarized (Ethan Schoonover), light + dark variants.
    Solarized,
    /// Nord (Arctic Ice Studio), dark-only.
    Nord,
}

/// A base16 palette: sixteen colors as `#RRGGBB` hex strings, base00 (background)
/// through base0F. The shell parses these into platform colors; invalid strings
/// are the shell's problem to tolerate, not the core's to prevent.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Base16Palette {
    pub base00: String,
    pub base01: String,
    pub base02: String,
    pub base03: String,
    pub base04: String,
    pub base05: String,
    pub base06: String,
    pub base07: String,
    pub base08: String,
    pub base09: String,
    pub base0a: String,
    pub base0b: String,
    pub base0c: String,
    pub base0d: String,
    pub base0e: String,
    pub base0f: String,
}

impl Base16Palette {
    /// Builds a palette from base00..base0F in order.
    fn from_hex(colors: [&str; 16]) -> Self {
        Base16Palette {
            base00: colors[0].to_string(),
            base01: colors[1].to_string(),
            base02: colors[2].to_string(),
            base03: colors[3].to_string(),
            base04: colors[4].to_string(),
            base05: colors[5].to_string(),
            base06: colors[6].to_string(),
            base07: colors[7].to_string(),
            base08: colors[8].to_string(),
            base09: colors[9].to_string(),
            base0a: colors[10].to_string(),
            base0b: colors[11].to_string(),
            base0c: colors[12].to_string(),
            base0d: colors[13].to_string(),
            base0e: colors[14].to_string(),
            base0f: colors[15].to_string(),
        }
    }
}

/// Read-only projection of the active theme for the shell. Carries both variants
/// plus enough metadata for the shell to resolve a concrete palette: pick `light`
/// or `dark` from `mode` + the OS scheme (respecting `has_dark_variant`), unless
/// `follows_system_colors` is set, in which case the shell uses native OS colors
/// and the palettes are unused.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ThemeView {
    pub id: ThemeId,
    pub name: String,
    pub mode: ThemeMode,
    /// When true, ignore the palettes and use the platform's semantic colors.
    pub follows_system_colors: bool,
    /// When false, the theme is single-variant: `light == dark` and `mode` is moot.
    pub has_dark_variant: bool,
    pub light: Base16Palette,
    pub dark: Base16Palette,
}

impl Default for ThemeView {
    fn default() -> Self {
        theme_view(ThemeId::default(), ThemeMode::default())
    }
}

/// Projects the selected theme + mode into a [`ThemeView`] for the shell.
pub fn theme_view(id: ThemeId, mode: ThemeMode) -> ThemeView {
    match id {
        ThemeId::System => ThemeView {
            id,
            name: "System".to_string(),
            mode,
            follows_system_colors: true,
            has_dark_variant: true,
            light: system_placeholder(),
            dark: system_placeholder(),
        },
        ThemeId::Solarized => ThemeView {
            id,
            name: "Solarized".to_string(),
            mode,
            follows_system_colors: false,
            has_dark_variant: true,
            light: solarized_light(),
            dark: solarized_dark(),
        },
        ThemeId::Nord => {
            let palette = nord();
            ThemeView {
                id,
                name: "Nord".to_string(),
                mode,
                follows_system_colors: false,
                has_dark_variant: false,
                light: palette.clone(),
                dark: palette,
            }
        }
    }
}

/// Neutral grayscale stand-in for the System theme, whose palettes the shell never
/// reads (it uses OS semantic colors). Present only so the projection is total.
fn system_placeholder() -> Base16Palette {
    Base16Palette::from_hex([
        "#ffffff", "#f2f2f7", "#e5e5ea", "#c7c7cc", "#8e8e93", "#000000", "#1c1c1e", "#000000",
        "#ff3b30", "#ff9500", "#ffcc00", "#34c759", "#5ac8fa", "#007aff", "#af52de", "#a2845e",
    ])
}

/// Solarized light: base00 is the lightest background, base07 the darkest
/// foreground. base08–base0F (the accent colors) are shared with the dark variant.
fn solarized_light() -> Base16Palette {
    Base16Palette::from_hex([
        "#fdf6e3", "#eee8d5", "#93a1a1", "#839496", "#657b83", "#586e75", "#073642", "#002b36",
        "#dc322f", "#cb4b16", "#b58900", "#859900", "#2aa198", "#268bd2", "#6c71c4", "#d33682",
    ])
}

/// Solarized dark: base00 is the darkest background, base07 the lightest
/// foreground — base00–base07 reversed from the light variant; accents shared.
fn solarized_dark() -> Base16Palette {
    Base16Palette::from_hex([
        "#002b36", "#073642", "#586e75", "#657b83", "#839496", "#93a1a1", "#eee8d5", "#fdf6e3",
        "#dc322f", "#cb4b16", "#b58900", "#859900", "#2aa198", "#268bd2", "#6c71c4", "#d33682",
    ])
}

/// Nord (dark-only).
fn nord() -> Base16Palette {
    Base16Palette::from_hex([
        "#2e3440", "#3b4252", "#434c5e", "#4c566a", "#d8dee9", "#e5e9f0", "#eceff4", "#8fbcbb",
        "#bf616a", "#d08770", "#ebcb8b", "#a3be8c", "#88c0d0", "#81a1c1", "#b48ead", "#5e81ac",
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_is_system_following_the_os() {
        let view = ThemeView::default();
        assert_eq!(view.id, ThemeId::System);
        assert_eq!(view.mode, ThemeMode::FollowSystem);
        assert!(
            view.follows_system_colors,
            "System must defer to OS semantic colors"
        );
    }

    #[test]
    fn solarized_is_two_variant_with_reversed_backgrounds() {
        let view = theme_view(ThemeId::Solarized, ThemeMode::FollowSystem);
        assert!(!view.follows_system_colors);
        assert!(view.has_dark_variant);
        // base00 (background) flips between variants; the accent (base0D) is shared.
        assert_ne!(view.light.base00, view.dark.base00);
        assert_eq!(view.light.base00, view.dark.base07);
        assert_eq!(view.light.base0d, view.dark.base0d);
    }

    #[test]
    fn nord_is_single_variant() {
        let view = theme_view(ThemeId::Nord, ThemeMode::FollowSystem);
        assert!(!view.has_dark_variant);
        assert_eq!(
            view.light, view.dark,
            "a single-variant theme has identical light and dark palettes"
        );
    }
}
