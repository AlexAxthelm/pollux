import Foundation

/// Waiting on state the Rust core owns. The shell can't await a core operation directly
/// (the core is driven by events and publishes a view), so the few places that need to hold
/// something open until the core is idle, like a pull-to-refresh spinner or a background
/// task, observe its view by polling.
enum Polling {
    /// How often `waitWhileBusy` re-reads the state. Short enough that a spinner doesn't
    /// linger after the work ends, long enough to be negligible work.
    static let interval: Duration = .milliseconds(100)

    /// Suspends while `isBusy` returns true, re-checking every `interval`, and returns as
    /// soon as it is false or the calling task is cancelled. Cancellation only ends the
    /// wait: it does not stop whatever the core is doing (see `Core.refreshStaleAndWait`
    /// for how a background task handles that).
    ///
    /// Main-actor isolated because `isBusy` reads the core's published view, which lives
    /// there. `interval` is a parameter only so tests don't have to wait a tenth of a
    /// second per poll.
    @MainActor
    static func waitWhileBusy(every interval: Duration = interval, _ isBusy: () -> Bool) async {
        while isBusy(), !Task.isCancelled {
            try? await Task.sleep(for: interval)
        }
    }
}
