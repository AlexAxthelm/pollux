import App
@testable import Pollux
import SwiftUI
import Testing

@Suite("Theme resolution")
struct ThemeTests {
    /// A palette whose background (base00) is `background`; the other slots are
    /// filled with `filler` since these tests only care about base00.
    private func makePalette(background: String, filler: String = "#808080") -> Base16Palette {
        Base16Palette(
            base00: background, base01: filler, base02: filler, base03: filler,
            base04: filler, base05: filler, base06: filler, base07: filler,
            base08: filler, base09: filler, base0a: filler, base0b: filler,
            base0c: filler, base0d: filler, base0e: filler, base0f: filler,
        )
    }

    private func makeTheme(
        hasDark: Bool,
        mode: ThemeMode,
        light: Base16Palette,
        dark: Base16Palette,
    ) -> ThemeView {
        ThemeView(
            id: .solarized,
            name: "Test",
            mode: mode,
            followsSystemColors: false,
            hasDarkVariant: hasDark,
            light: light,
            dark: dark,
        )
    }

    // MARK: - preferredColorScheme

    // A single-variant dark-only theme (e.g. Nord) must force dark chrome even
    // under FollowSystem, so system UI doesn't render light over a dark palette.
    @Test func singleVariantDarkThemePinsDarkScheme() {
        let dark = makePalette(background: "#002b36")
        let view = makeTheme(hasDark: false, mode: .followSystem, light: dark, dark: dark)
        #expect(view.preferredColorScheme == .dark)
    }

    @Test func singleVariantLightThemePinsLightScheme() {
        let light = makePalette(background: "#fdf6e3")
        let view = makeTheme(hasDark: false, mode: .followSystem, light: light, dark: light)
        #expect(view.preferredColorScheme == .light)
    }

    @Test func dualVariantFollowSystemDefersToOS() {
        let view = makeTheme(
            hasDark: true, mode: .followSystem,
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
        )
        #expect(view.preferredColorScheme == nil)
    }

    @Test func modeLightForcesLightScheme() {
        let view = makeTheme(
            hasDark: true, mode: .light,
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
        )
        #expect(view.preferredColorScheme == .light)
    }

    @Test func modeDarkForcesDarkScheme() {
        let view = makeTheme(
            hasDark: true, mode: .dark,
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
        )
        #expect(view.preferredColorScheme == .dark)
    }

    // MARK: - palette(for:)

    @Test func dualVariantFollowSystemPicksPaletteByOSScheme() {
        let view = makeTheme(
            hasDark: true, mode: .followSystem,
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
        )
        #expect(view.palette(for: .light).base00 == "#ffffff")
        #expect(view.palette(for: .dark).base00 == "#000000")
    }

    @Test func pinnedModeIgnoresOSScheme() {
        let darkPinned = makeTheme(
            hasDark: true, mode: .dark,
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
        )
        // OS is light, but the dark palette is applied.
        #expect(darkPinned.palette(for: .light).base00 == "#000000")

        let lightPinned = makeTheme(
            hasDark: true, mode: .light,
            light: makePalette(background: "#ffffff"), dark: makePalette(background: "#000000"),
        )
        // OS is dark, but the light palette is applied.
        #expect(lightPinned.palette(for: .dark).base00 == "#ffffff")
    }

    @Test func singleVariantAlwaysUsesLightPalette() {
        let only = makePalette(background: "#2e3440")
        let view = makeTheme(
            hasDark: false, mode: .light,
            light: only, dark: makePalette(background: "#ffffff"),
        )
        #expect(view.palette(for: .dark).base00 == "#2e3440")
    }

    // MARK: - isDarkBackground

    @Test func luminanceClassifiesBackgrounds() {
        #expect(makePalette(background: "#002b36").isDarkBackground)
        #expect(makePalette(background: "#2e3440").isDarkBackground)
        #expect(makePalette(background: "#0A0A0A").isDarkBackground) // uppercase hex parses
        #expect(!makePalette(background: "#fdf6e3").isDarkBackground)
        #expect(!makePalette(background: "#ffffff").isDarkBackground)
    }

    // Malformed hex (non-hex chars, a stray sign, or wrong length) is treated as
    // light — the safer default for system chrome, not a silently wrong color.
    @Test func malformedBackgroundIsNotDark() {
        #expect(!makePalette(background: "nothex").isDarkBackground)
        #expect(!makePalette(background: "+12345").isDarkBackground) // sign prefix rejected
        #expect(!makePalette(background: "#12345").isDarkBackground) // too short
    }
}
