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
            await core.runBackgroundRefresh()
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
                // The lifecycle logic lives in `Core` so it can be tested; this only maps
                // the scene phase onto it. `initial: true` covers cold launch.
                switch phase {
                case .active:
                    core.appBecameActive()
                case .background:
                    core.appEnteredBackground()
                default:
                    break
                }
            }
    }
}
