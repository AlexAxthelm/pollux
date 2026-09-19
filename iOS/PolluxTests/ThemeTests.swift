import App
@testable import Pollux
import SwiftUI
import Testing

@Suite("Theme resolution")
struct ThemeTests {
    /// A palette whose background (base00) is `background`; the other slots are
    /// filled with `filler` since these tests only care about base00 identity.
    private func makePalette(background: String, filler: String = "#808080") -> Base16Palette {
        Base16Palette(
            base00: background, base01: filler, base02: filler, base03: filler,
            base04: filler, base05: filler, base06: filler, base07: filler,
            base08: filler, base09: filler, base0a: filler, base0b: filler,
            base0c: filler, base0d: filler, base0e: filler, base0f: filler,
        )
    }

    private func makeTheme(
        light: Base16Palette?,
        dark: Base16Palette?,
        mode: ThemeMode = .followSystem,
    ) -> ThemeView {
        ThemeView(
            id: .solarized,
            name: "Test",
            mode: mode,
            light: light,
            dark: dark,
        )
    }

    // MARK: - preferredColorScheme

    /// A single-variant theme declares its appearance by the slot it fills, and the
    /// scheme is pinned to it regardless of `mode` — so dark-only Nord forces dark
    /// chrome even under FollowSystem.
    @Test func darkOnlyThemePinsDarkScheme() {
        let view = makeTheme(light: nil, dark: makePalette(background: "#2e3440"))
        #expect(view.preferredColorScheme == .dark)
    }

    @Test func lightOnlyThemePinsLightScheme() {
        let view = makeTheme(light: makePalette(background: "#fdf6e3"), dark: nil)
        #expect(view.preferredColorScheme == .light)
    }

    @Test func dualVariantFollowSystemDefersToOS() {
        let view = makeTheme(
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
            mode: .followSystem,
        )
        #expect(view.preferredColorScheme == nil)
    }

    @Test func dualVariantModeLightForcesLight() {
        let view = makeTheme(
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
            mode: .light,
        )
        #expect(view.preferredColorScheme == .light)
    }

    @Test func dualVariantModeDarkForcesDark() {
        let view = makeTheme(
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
            mode: .dark,
        )
        #expect(view.preferredColorScheme == .dark)
    }

    @Test func systemHonorsMode() {
        let followOS = makeTheme(light: nil, dark: nil, mode: .followSystem)
        #expect(followOS.preferredColorScheme == nil)

        let forcedDark = makeTheme(light: nil, dark: nil, mode: .dark)
        #expect(forcedDark.preferredColorScheme == .dark)
    }

    // MARK: - palette(for:)

    @Test func dualVariantFollowSystemPicksPaletteByOSScheme() {
        let view = makeTheme(
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
            mode: .followSystem,
        )
        #expect(view.palette(for: .light)?.base00 == "#ffffff")
        #expect(view.palette(for: .dark)?.base00 == "#000000")
    }

    @Test func pinnedModeIgnoresOSScheme() {
        let darkPinned = makeTheme(
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
            mode: .dark,
        )
        // OS is light, but the dark palette is applied.
        #expect(darkPinned.palette(for: .light)?.base00 == "#000000")

        let lightPinned = makeTheme(
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
            mode: .light,
        )
        // OS is dark, but the light palette is applied.
        #expect(lightPinned.palette(for: .dark)?.base00 == "#ffffff")
    }

    @Test func darkOnlyThemeAlwaysUsesDarkPalette() {
        let view = makeTheme(light: nil, dark: makePalette(background: "#2e3440"))
        #expect(view.palette(for: .light)?.base00 == "#2e3440")
        #expect(view.palette(for: .dark)?.base00 == "#2e3440")
    }

    @Test func lightOnlyThemeAlwaysUsesLightPalette() {
        let view = makeTheme(light: makePalette(background: "#fdf6e3"), dark: nil)
        #expect(view.palette(for: .dark)?.base00 == "#fdf6e3")
    }

    @Test func systemThemeHasNoPalette() {
        let view = makeTheme(light: nil, dark: nil)
        #expect(view.palette(for: .light) == nil)
        #expect(view.palette(for: .dark) == nil)
    }

    // MARK: - ThemeColors.resolve

    @Test func resolveUsesSystemColorsWhenThemeHasNoPalette() {
        let system = makeTheme(light: nil, dark: nil)
        #expect(ThemeColors.resolve(system, colorScheme: .light) == .system)
        #expect(ThemeColors.resolve(system, colorScheme: .dark) == .system)
    }

    @Test func resolveBuildsColorsFromTheSelectedPalette() {
        let light = makePalette(background: "#ffffff")
        let dark = makePalette(background: "#000000")
        let view = makeTheme(light: light, dark: dark, mode: .followSystem)
        #expect(ThemeColors.resolve(view, colorScheme: .dark) == .from(palette: dark))
        #expect(ThemeColors.resolve(view, colorScheme: .light) == .from(palette: light))
    }

    // MARK: - Base16.rgb parsing

    @Test func base16ParsesValidHex() {
        #expect(Base16.rgb("#0a1b2c") == 0x0A1B2C)
        #expect(Base16.rgb("0A1B2C") == 0x0A1B2C) // no leading '#', uppercase
        #expect(Base16.rgb("#ffffff") == 0xFFFFFF)
    }

    @Test func base16RejectsMalformedHex() {
        #expect(Base16.rgb("nothex") == nil) // non-hex characters
        #expect(Base16.rgb("+12345") == nil) // leading sign UInt32 would otherwise accept
        #expect(Base16.rgb("#12345") == nil) // too short
        #expect(Base16.rgb("#1234567") == nil) // too long
    }
}
