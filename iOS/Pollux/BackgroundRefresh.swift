import BackgroundTasks
import Foundation

/// Best-effort background feed refresh via `BGAppRefreshTask`.
///
/// iOS decides when (and whether) the task actually runs: `earliestBeginDate` is only a
/// lower bound, and the system may run it much later or not at all (Low Power Mode,
/// Background App Refresh off, an app the user rarely opens or has force-quit). So this
/// is a bonus on top of the foreground refresh, not a schedule. A run gets roughly 30
/// seconds; the core's serial queue refreshes as many due feeds as fit and leaves the rest
/// for the next foreground or wake-up. Metadata only — no downloads.
enum BackgroundRefresh {
    /// Must match `BGTaskSchedulerPermittedIdentifiers` in `iOS/project.yml`.
    static let identifier = "com.example.pollux.refresh"

    /// How soon after scheduling the system may first run the task. Deliberately shorter
    /// than the 12h refresh interval: the core only fetches feeds that are actually due,
    /// so an early wake costs almost nothing, while a longer request would push the
    /// system's real (later) run well past the interval.
    static let minimumInterval: TimeInterval = 3 * 3600

    static func earliestBeginDate(from now: Date = Date()) -> Date {
        now.addingTimeInterval(minimumInterval)
    }

    /// Asks the system for a future wake-up. Replaces any pending request for the same
    /// identifier, so calling it repeatedly (every backgrounding, every run) is safe.
    @MainActor
    static func schedule() {
        let request = BGAppRefreshTaskRequest(identifier: identifier)
        request.earliestBeginDate = earliestBeginDate()
        do {
            try BGTaskScheduler.shared.submit(request)
        } catch {
            // Expected on the simulator and when Background App Refresh is unavailable;
            // foreground refresh still works, so this isn't worth surfacing.
            NSLog("BackgroundRefresh: could not schedule: \(error.localizedDescription)")
        }
    }

    /// The task body. Reschedules first so the chain survives a run cut short by the
    /// system, then refreshes whatever is due until the queue drains or the task is
    /// cancelled at expiry.
    static func run(core: Core) async {
        await MainActor.run { schedule() }
        await core.refreshStaleAndWait()
    }
}
