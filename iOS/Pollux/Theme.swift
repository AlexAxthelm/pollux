import App
import SwiftUI
import UIKit

// Resolves the core's active `ThemeView` into the concrete colors the UI reads.
//
// The core owns palette *data* (base16 tokens; see `docs/features/theme.md` and
// `docs/features/theme-semantic-mapping.md`); this layer is the shell's half:
// it maps base16 tokens to SwiftUI `Color`s, exposes them as a semantic set via
// the environment (`\.themeColors`), and is injected once at the app root.
//
// The `System` theme (`followsSystemColors`) resolves to the platform's own
// semantic colors, so it reproduces the app's appearance before theming landed.

// MARK: - Semantic color set

/// The semantic roles the UI paints with. Only the base16 tokens actually used by
/// the app are surfaced (per the spec); the mapping is documented in
/// `docs/features/theme-semantic-mapping.md`.
struct ThemeColors {
    /// Screen background — base00.
    let background: Color
    /// Elevated/secondary surface (cards, placeholders) — base01.
    let secondaryBackground: Color
    /// Default foreground / body text — base05.
    let text: Color
    /// De-emphasized text (captions, metadata, inactive) — base03.
    let secondaryText: Color
    /// Accent / links / tint — base0D.
    let accent: Color
    /// Errors — base08.
    let error: Color
    /// Success — base0B.
    let success: Color
    /// Warnings — base0A.
    let warning: Color

    /// The platform's own semantic colors — the appearance the app had before
    /// theming. Used for the `System` theme, and as the environment default.
    static let system = ThemeColors(
        background: Color(uiColor: .systemBackground),
        secondaryBackground: Color(uiColor: .secondarySystemBackground),
        text: .primary,
        secondaryText: .secondary,
        accent: .accentColor,
        error: .red,
        success: .green,
        warning: .orange,
    )

    /// Builds the semantic set from a resolved base16 palette.
    init(palette: Base16Palette) {
        background = Color(base16: palette.base00)
        secondaryBackground = Color(base16: palette.base01)
        text = Color(base16: palette.base05)
        secondaryText = Color(base16: palette.base03)
        accent = Color(base16: palette.base0d)
        error = Color(base16: palette.base08)
        success = Color(base16: palette.base0b)
        warning = Color(base16: palette.base0a)
    }

    private init(
        background: Color,
        secondaryBackground: Color,
        text: Color,
        secondaryText: Color,
        accent: Color,
        error: Color,
        success: Color,
        warning: Color,
    ) {
        self.background = background
        self.secondaryBackground = secondaryBackground
        self.text = text
        self.secondaryText = secondaryText
        self.accent = accent
        self.error = error
        self.success = success
        self.warning = warning
    }

    /// Resolves the active theme for the current OS appearance. Falls back to the
    /// system colors for the System theme, which carries no palette.
    static func resolve(_ theme: ThemeView, colorScheme: ColorScheme) -> ThemeColors {
        guard !theme.followsSystemColors, let palette = theme.palette(for: colorScheme) else {
            return .system
        }
        return ThemeColors(palette: palette)
    }
}

// MARK: - ThemeView resolution

extension ThemeView {
    /// The palette to apply given the pinned mode and the OS appearance, or nil for
    /// a theme that carries none (System, which uses OS colors). A single-variant
    /// theme always uses `light` (which equals `dark`).
    func palette(for colorScheme: ColorScheme) -> Base16Palette? {
        guard hasDarkVariant else { return light }
        switch mode {
        case .light: return light
        case .dark: return dark
        case .followSystem: return colorScheme == .dark ? dark : light
        }
    }

    /// The color scheme to force on the app, or nil to follow the OS. Applies to
    /// the `System` theme too (its semantic colors then resolve light or dark).
    var preferredColorScheme: ColorScheme? {
        // A single-variant theme (e.g. Nord, dark-only) offers no light/dark
        // choice, so `mode` is moot. Pin the scheme to the palette's own luminance
        // so system chrome (status bar, controls) matches it — otherwise a
        // dark-only theme under FollowSystem would draw light chrome on a light OS.
        guard hasDarkVariant else {
            return (light?.isDarkBackground ?? false) ? .dark : .light
        }
        switch mode {
        case .light: return .light
        case .dark: return .dark
        case .followSystem: return nil
        }
    }
}

extension Base16Palette {
    /// Whether base00 (the background) reads as dark, by perceived luminance.
    /// Malformed hex is treated as light (the safer default for system chrome).
    var isDarkBackground: Bool {
        guard let rgb = base16RGB(base00) else { return false }
        let red = Double((rgb >> 16) & 0xFF)
        let green = Double((rgb >> 8) & 0xFF)
        let blue = Double(rgb & 0xFF)
        // Rec. 601 luma on a 0...255 scale; below the midpoint is a dark background.
        return (0.299 * red + 0.587 * green + 0.114 * blue) < 127.5
    }
}

// MARK: - Hex parsing

/// Parses a base16 `#RRGGBB` (or `RRGGBB`) string into its 24-bit RGB value, or
/// nil if malformed. Shared by `Color(base16:)` and the luminance check. Requires
/// exactly six hex digits: `UInt32(_:radix:)` alone would accept a leading sign
/// (e.g. "+12345"), so the character set is validated explicitly.
private func base16RGB(_ hex: String) -> UInt32? {
    let digits = hex.hasPrefix("#") ? String(hex.dropFirst()) : hex
    guard digits.count == 6, digits.allSatisfy(\.isHexDigit) else {
        return nil
    }
    return UInt32(digits, radix: 16)
}

extension Color {
    /// Parses a base16 `#RRGGBB` (or `RRGGBB`) hex string in the sRGB space.
    /// Falls back to a neutral gray on malformed input — the built-in palettes are
    /// always valid, so this only guards against a future bad custom value.
    init(base16 hex: String) {
        guard let rgb = base16RGB(hex) else {
            self = .gray
            return
        }
        self = Color(
            .sRGB,
            red: Double((rgb >> 16) & 0xFF) / 255,
            green: Double((rgb >> 8) & 0xFF) / 255,
            blue: Double(rgb & 0xFF) / 255,
            opacity: 1,
        )
    }
}

extension EnvironmentValues {
    /// The active theme's semantic colors. Injected at the app root; read by views
    /// with `@Environment(\.themeColors)`.
    @Entry var themeColors: ThemeColors = .system
}
