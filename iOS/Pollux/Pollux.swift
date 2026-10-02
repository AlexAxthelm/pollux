import App
import SwiftUI

@main
struct PolluxApp: App {
    @StateObject private var core = Core()

    var body: some Scene {
        WindowGroup {
            RootView(core: core)
        }
    }
}

/// Applies the core's active theme globally, then hosts the app. Reads the OS
/// appearance so a `followSystem` theme picks light/dark correctly, and resolves
/// the base16 palette into the `\.themeColors` the views paint with.
private struct RootView: View {
    @ObservedObject var core: Core
    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.scenePhase) private var scenePhase

    private var theme: ThemeView {
        core.view.theme
    }

    private var colors: ThemeColors {
        ThemeColors.resolve(theme, colorScheme: colorScheme)
    }

    var body: some View {
        ContentView(core: core)
            .environment(\.themeColors, colors)
            .tint(colors.accent)
            .background(colors.background.ignoresSafeArea())
            .preferredColorScheme(theme.preferredColorScheme)
            .onChange(of: scenePhase, initial: true) { _, phase in
                // Foreground auto-refresh: the core decides which feeds are due (12h
                // interval, honouring backoff), and holds the request if the library
                // hasn't loaded yet, so firing on every activation incl. cold launch is safe.
                if phase == .active {
                    core.update(.refreshStale)
                }
            }
    }
}
