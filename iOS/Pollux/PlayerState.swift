import App
import Foundation

/// The active player, kept apart from `Core.view` on purpose.
///
/// The player changes every second while playing (the position). Everything that observes
/// `Core` is re-evaluated whenever it publishes, so if the player lived in `Core.view`, a
/// tick would make SwiftUI re-evaluate the whole episode list behind it. Here only the views
/// that show the player (the mini-player and the full player) observe it, and the list only
/// hears from `Core` when its own data changes.
@MainActor
final class PlayerState: ObservableObject {
    @Published private(set) var player: PlayerView?

    /// Replaces the player, publishing only if it actually changed.
    func update(_ new: PlayerView?) {
        if new != player {
            player = new
        }
    }
}
