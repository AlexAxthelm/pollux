import App
import SwiftUI

@main
struct PolluxApp: App {
    @StateObject private var core = Core()

    var body: some Scene {
        WindowGroup {
            RootView(core: core)
        }
        .backgroundTask(.appRefresh(BackgroundRefresh.identifier)) { [core] in
            await BackgroundRefresh.run(core: core)
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
                switch phase {
                case .active:
                    core.update(.refreshStale)
                    // Resume interrupted downloads only once the app is actually in the
                    // foreground. A background launch (the refresh task) never reaches
                    // `.active`, so it can't start a download inside its short window.
                    // The core ignores every activation after the first.
                    core.update(.resumePendingDownloads)
                case .background:
                    // Queue the next best-effort background wake-up.
                    BackgroundRefresh.schedule()
                default:
                    break
                }
            }
    }
}
