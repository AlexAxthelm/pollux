import App
import Combine
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
                MiniPlayerHost(
                    playerState: core.playerState,
                    isPlayerOpen: showPlayer,
                    onOpen: { showPlayer = true },
                    onTogglePlay: { core.update(.togglePlay) },
                )
            }
            .fullScreenCover(isPresented: $showPlayer) {
                FullPlayerHost(
                    playerState: core.playerState,
                    colors: colors,
                    theme: theme,
                    send: { core.update($0) },
                    onHide: { showPlayer = false },
                    onGoToSource: { goToSource(of: $0) },
                )
            }
            .environment(\.themeColors, colors)
            .tint(colors.accent)
            .background(colors.background.ignoresSafeArea())
            .preferredColorScheme(theme.preferredColorScheme)
            // Subscribed to rather than observed: this view must not be re-evaluated every
            // second just because the playing position changed.
            .onReceive(
                core.playerState.$player.map { $0?.episodeId }.removeDuplicates(),
            ) { episodeId in
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

/// The mini-player. Observes the player state itself so the root view, and the list under
/// it, are not re-evaluated every time the playing position changes.
private struct MiniPlayerHost: View {
    @ObservedObject var playerState: PlayerState
    let isPlayerOpen: Bool
    let onOpen: () -> Void
    let onTogglePlay: () -> Void

    var body: some View {
        // Hidden entirely when nothing is active, and while the full player is up.
        if let player = playerState.player, !isPlayerOpen {
            MiniPlayerBar(player: player, onOpen: onOpen, onTogglePlay: onTogglePlay)
        }
    }
}

/// The full-screen player, observing the player state for the same reason.
private struct FullPlayerHost: View {
    @ObservedObject var playerState: PlayerState
    let colors: ThemeColors
    let theme: ThemeView
    let send: (Event) -> Void
    let onHide: () -> Void
    let onGoToSource: (PlayerView) -> Void

    var body: some View {
        if let player = playerState.player {
            PlayerScreen(
                player: player,
                send: send,
                onHide: onHide,
                onGoToSource: { onGoToSource(player) },
            )
            // A cover doesn't reliably inherit the theme applied below it.
            .environment(\.themeColors, colors)
            .tint(colors.accent)
            .preferredColorScheme(theme.preferredColorScheme)
        }
    }
}
