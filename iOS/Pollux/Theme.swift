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

    /// Resolves the active theme for the current OS appearance.
    static func resolve(_ theme: ThemeView, colorScheme: ColorScheme) -> ThemeColors {
        theme.followsSystemColors
            ? .system
            : ThemeColors(palette: theme.palette(for: colorScheme))
    }
}

// MARK: - ThemeView resolution

extension ThemeView {
    /// Picks the palette to apply given the pinned mode and the OS appearance.
    /// A single-variant theme always uses `light` (which equals `dark`).
    func palette(for colorScheme: ColorScheme) -> Base16Palette {
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
        switch mode {
        case .light: .light
        case .dark: .dark
        case .followSystem: nil
        }
    }
}

// MARK: - Hex parsing

extension Color {
    /// Parses a base16 `#RRGGBB` (or `RRGGBB`) hex string in the sRGB space.
    /// Falls back to a neutral gray on malformed input — the built-in palettes are
    /// always valid, so this only guards against a future bad custom value.
    init(base16 hex: String) {
        let digits = hex.hasPrefix("#") ? String(hex.dropFirst()) : hex
        guard digits.count == 6, let value = UInt32(digits, radix: 16) else {
            self = .gray
            return
        }
        self = Color(
            .sRGB,
            red: Double((value >> 16) & 0xFF) / 255.0,
            green: Double((value >> 8) & 0xFF) / 255.0,
            blue: Double(value & 0xFF) / 255.0,
            opacity: 1,
        )
    }
}

extension EnvironmentValues {
    // The active theme's semantic colors. Injected at the app root; read by views
    // with `@Environment(\.themeColors)`.
    @Entry var themeColors: ThemeColors = .system
}
