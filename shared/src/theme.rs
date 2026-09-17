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
/// A single-variant theme (only one of `light`/`dark` present, e.g. Nord) ignores
/// this — the choice is meaningful only when both variants exist.
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

impl ThemeId {
    /// Every built-in theme, in display order — the single place that enumerates
    /// them, for the Settings selector and for tests that sweep all themes. Adding
    /// a theme means a new variant, its arm in [`theme_view`], and an entry here.
    pub const ALL: [ThemeId; 3] = [ThemeId::System, ThemeId::Solarized, ThemeId::Nord];
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

/// Read-only projection of the active theme for the shell. Carries the palettes
/// plus enough metadata for the shell to resolve a concrete one: for a dual-variant
/// theme, pick `light` or `dark` from `mode` + the OS scheme; for a single-variant
/// theme, use the one present slot; for System (both palettes `None`), use the
/// platform's native colors.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ThemeView {
    pub id: ThemeId,
    pub name: String,
    pub mode: ThemeMode,
    /// The light and dark palettes, and the sole signal of a theme's structure:
    /// both `None` means System (use OS colors); both `Some` is a dual-variant
    /// theme; exactly one `Some` is a single-variant theme (e.g. Nord is dark-only)
    /// that declares its appearance by which slot it fills, so the shell can pin the
    /// scheme to it — no light/dark detection needed. Kept out of the projection
    /// when unused rather than shipped as dead placeholder data.
    pub light: Option<Base16Palette>,
    pub dark: Option<Base16Palette>,
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
            light: None,
            dark: None,
        },
        ThemeId::Solarized => ThemeView {
            id,
            name: "Solarized".to_string(),
            mode,
            light: Some(solarized_light()),
            dark: Some(solarized_dark()),
        },
        ThemeId::Nord => ThemeView {
            id,
            name: "Nord".to_string(),
            mode,
            // Dark-only: only the dark slot is filled, which declares its appearance.
            light: None,
            dark: Some(nord()),
        },
    }
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

    /// sRGB channel (0–255) to linear light, per WCAG.
    fn linear(channel: u8) -> f64 {
        let s = f64::from(channel) / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }

    /// WCAG relative luminance of a `#RRGGBB` hex string.
    fn relative_luminance(hex: &str) -> f64 {
        let h = hex.trim_start_matches('#');
        let parse = |s: &str| u8::from_str_radix(s, 16).unwrap_or(0);
        0.2126 * linear(parse(&h[0..2]))
            + 0.7152 * linear(parse(&h[2..4]))
            + 0.0722 * linear(parse(&h[4..6]))
    }

    /// WCAG contrast ratio between two `#RRGGBB` colors (1.0–21.0).
    fn contrast(a: &str, b: &str) -> f64 {
        let (la, lb) = (relative_luminance(a), relative_luminance(b));
        let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }

    #[test]
    fn built_in_text_tokens_stay_legible_on_the_background() {
        // Sweep every built-in theme from the single source of truth (`ThemeId::ALL`)
        // and check each palette it ships (both variants of a dual theme; the one
        // slot of a single-variant theme; System has none, so it's skipped). Adding
        // a theme automatically extends this test.
        //
        // The shell paints `text` with base05 and `secondaryText` with base04 (see
        // docs/features/theme-semantic-mapping.md). Primary text must clear WCAG AA
        // (4.5:1); secondary text must clear the 3:1 large-text/UI bar — canonical
        // palettes can't always reach 4.5:1 for de-emphasized text (that gap is
        // documented, with more accessible themes planned). base03 (the "comments"
        // color) must never back text: it fails even 3:1.
        for id in ThemeId::ALL {
            let view = theme_view(id, ThemeMode::FollowSystem);
            for palette in view.light.iter().chain(view.dark.iter()) {
                let text = contrast(&palette.base00, &palette.base05);
                assert!(
                    text >= 4.5,
                    "{}: text (base05) is only {text:.2}:1 on base00",
                    view.name
                );
                let secondary = contrast(&palette.base00, &palette.base04);
                assert!(
                    secondary >= 3.0,
                    "{}: secondaryText (base04) is only {secondary:.2}:1 on base00",
                    view.name
                );
            }
        }
    }

    #[test]
    fn default_theme_is_system_following_the_os() {
        let view = ThemeView::default();
        assert_eq!(view.id, ThemeId::System);
        assert_eq!(view.mode, ThemeMode::FollowSystem);
        assert!(
            view.light.is_none() && view.dark.is_none(),
            "System defers to OS colors: it ships no palettes"
        );
    }

    #[test]
    fn solarized_is_two_variant_with_reversed_backgrounds() {
        let view = theme_view(ThemeId::Solarized, ThemeMode::FollowSystem);
        let (Some(light), Some(dark)) = (view.light, view.dark) else {
            panic!("a dual-variant theme must carry both variants");
        };
        // base00 (background) flips between variants; the accent (base0D) is shared.
        assert_ne!(light.base00, dark.base00);
        assert_eq!(light.base00, dark.base07);
        assert_eq!(light.base0d, dark.base0d);
    }

    #[test]
    fn nord_is_dark_only() {
        let view = theme_view(ThemeId::Nord, ThemeMode::FollowSystem);
        assert!(
            view.light.is_none(),
            "Nord is dark-only: it fills only the dark slot"
        );
        assert!(view.dark.is_some());
    }

    #[test]
    fn system_carries_no_palette_data() {
        let view = theme_view(ThemeId::System, ThemeMode::FollowSystem);
        assert!(
            view.light.is_none() && view.dark.is_none(),
            "System uses OS colors, so it must not ship placeholder palettes"
        );
    }
}
