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
    @State private var showPlayer = false
    /// The library's navigation stack, owned here so the player can jump back to the
    /// podcast it is playing from.
    @State private var path = NavigationPath()

    private var theme: ThemeView {
        core.view.theme
    }

    private var colors: ThemeColors {
        ThemeColors.resolve(theme, colorScheme: colorScheme)
    }

    var body: some View {
        ContentView(core: core, path: $path)
            // Hidden entirely when nothing is active, and while the full player is up.
            .safeAreaInset(edge: .bottom, spacing: 0) {
                if let player = core.view.player, !showPlayer {
                    MiniPlayerBar(
                        player: player,
                        onOpen: { showPlayer = true },
                        onTogglePlay: { core.update(.togglePlay) },
                    )
                }
            }
            .fullScreenCover(isPresented: $showPlayer) {
                fullPlayer
            }
            .environment(\.themeColors, colors)
            .tint(colors.accent)
            .background(colors.background.ignoresSafeArea())
            .preferredColorScheme(theme.preferredColorScheme)
            .onChange(of: core.view.player?.episodeId) { _, episodeId in
                // The episode finished (or was cleared): nothing left to show.
                if episodeId == nil {
                    showPlayer = false
                }
            }
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

    @ViewBuilder private var fullPlayer: some View {
        if let player = core.view.player {
            PlayerScreen(
                player: player,
                send: { core.update($0) },
                onHide: { showPlayer = false },
                onGoToSource: { goToSource(of: player) },
            )
            // A cover doesn't reliably inherit the theme applied below it.
            .environment(\.themeColors, colors)
            .tint(colors.accent)
            .preferredColorScheme(theme.preferredColorScheme)
        }
    }

    /// Dismisses the player and shows what playback was started from. A subscription is
    /// the only source for now; a playlist will get its own case here.
    private func goToSource(of player: PlayerView) {
        switch player.source {
        case let .subscription(id):
            guard let subscription = core.view.library.subscriptions
                .first(where: { $0.id == id })
            else { return }
            showPlayer = false
            path = NavigationPath()
            path.append(subscription)
        }
    }
}
