import App
import Foundation
import Testing

@testable import Pollux

// These drive a real `Core` and the real Rust core behind it, against a temporary database
// and a stubbed network. The Rust tests cover what the core *decides*; these cover that
// the shell *sends it the right events* and carries its requests out: the activation and
// background lifecycle, the background task's wait and cancellation, and that a background
// launch doesn't start downloads.

// MARK: - Rig

@MainActor
private final class Counter {
    var value = 0
}

@MainActor
private struct Rig {
    let core: Core
    let db: DatabaseManager
    let server: StubServer
    let scheduled: Counter
}

@MainActor
private func makeRig(seed: (DatabaseManager, StubServer) async throws -> Void = { _, _ in }) async throws -> Rig {
    let server = StubServer()
    let root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
    let db = try DatabaseManager(path: root.appendingPathExtension("sqlite").path)
    let downloads = try DownloadManager(storageRoot: root, sessionConfiguration: server.configuration())
    try await seed(db, server) // before the Core, whose launch load must see it
    let scheduled = Counter()
    let core = Core(
        db: db, downloads: downloads, feedSession: server.session(),
        scheduleBackgroundRefresh: { scheduled.value += 1 },
    )
    return Rig(core: core, db: db, server: server, scheduled: scheduled)
}

private let longAgo: Int64 = 1_000
private func now() -> Int64 { Int64(Date().timeIntervalSince1970) }

/// A subscription that was last refreshed long ago, so it is due, with a stored ETag.
private func dueSubscription(_ server: StubServer, id: String, title: String? = nil) -> Subscription {
    Subscription(
        id: id, feedUrl: server.url("/\(id).rss"), title: title ?? id, artworkUrl: nil, description: nil,
        lastRefreshed: longAgo, createdAt: longAgo, etag: "\"v1-\(id)\"", lastModified: nil,
        lastRefreshError: nil, retryAfterUntil: nil,
    )
}

private func notModified() -> StubServer.Reply {
    .init(status: 304)
}

private struct TimedOut: Error {}

/// Polls `condition` until it holds, failing the test if it doesn't within `timeout`.
@MainActor
private func waitUntil(timeout: Duration = .seconds(15), _ condition: () async throws -> Bool) async throws {
    let deadline = ContinuousClock.now + timeout
    while try await !condition() {
        if ContinuousClock.now > deadline { throw TimedOut() }
        try await Task.sleep(for: .milliseconds(10))
    }
}

/// Returns once the core has loaded the library and has nothing queued or in flight.
@MainActor
private func waitForIdle(_ core: Core) async {
    await Polling.waitWhileBusy(every: .milliseconds(5)) {
        core.view.library.loading || core.view.library.refreshing
    }
}

private func lastRefreshed(_ db: DatabaseManager, _ id: String) async throws -> Int64? {
    guard case let .subscription(sub) = try await db.execute(.getSubscription(id: id)) else { return nil }
    return sub.lastRefreshed
}

// MARK: - Tests

@Suite("Core wiring")
@MainActor
struct CoreWiringTests {
    // MARK: Foreground

    @Test(.timeLimit(.minutes(1)))
    func becomingActiveRefreshesDueFeedsUsingTheirStoredValidators() async throws {
        let rig = try await makeRig { db, server in
            try await db.execute(.upsertSubscription(dueSubscription(server, id: "a")))
            server.reply("/a.rss", notModified())
        }

        rig.core.appBecameActive()
        await waitForIdle(rig.core)

        let requests = rig.server.requests
        #expect(requests.map(\.path) == ["/a.rss"])
        #expect(requests.first?.headers["If-None-Match"] == "\"v1-a\"", "the stored ETag is replayed")
        try await waitUntil { try await lastRefreshed(rig.db, "a") ?? 0 > longAgo }
    }

    @Test(.timeLimit(.minutes(1)))
    func activationBeforeTheLibraryLoadsIsHeldAndHonouredNotDropped() async throws {
        // The cold-launch race: `appBecameActive` can run before the first library load has
        // returned. The core must hold the request rather than judge an empty library.
        let rig = try await makeRig { db, server in
            try await db.execute(.upsertSubscription(dueSubscription(server, id: "a")))
            server.reply("/a.rss", notModified())
        }
        #expect(rig.core.view.library.subscriptions.isEmpty, "the launch load hasn't landed yet")

        rig.core.appBecameActive()
        await waitForIdle(rig.core)

        #expect(rig.server.requestedPaths == ["/a.rss"])
    }

    @Test(.timeLimit(.minutes(1)))
    func aFeedThatIsNotDueIsNotFetchedOnActivation() async throws {
        let rig = try await makeRig { db, server in
            var fresh = dueSubscription(server, id: "a")
            fresh.lastRefreshed = now()
            try await db.execute(.upsertSubscription(fresh))
            server.reply("/a.rss", notModified())
        }

        rig.core.appBecameActive()
        await waitForIdle(rig.core)

        #expect(rig.server.requests.isEmpty)
    }

    // MARK: Background

    @Test func enteringTheBackgroundSchedulesAnotherWakeUp() async throws {
        let rig = try await makeRig()

        rig.core.appEnteredBackground()
        #expect(rig.scheduled.value == 1)
        rig.core.appEnteredBackground()
        #expect(rig.scheduled.value == 2, "every backgrounding re-requests")
    }

    @Test(.timeLimit(.minutes(1)))
    func theBackgroundTaskWaitsForTheLibraryThenRefreshesEveryDueFeedInOrder() async throws {
        let rig = try await makeRig { db, server in
            try await db.execute(.upsertSubscription(dueSubscription(server, id: "b", title: "Bravo")))
            try await db.execute(.upsertSubscription(dueSubscription(server, id: "a", title: "Alpha")))
            server.reply("/a.rss", notModified())
            server.reply("/b.rss", notModified())
        }

        await rig.core.runBackgroundRefresh() // straight from launch: library not loaded yet

        #expect(rig.server.requestedPaths == ["/a.rss", "/b.rss"], "library (title) order, all of them")
        #expect(rig.core.view.library.refreshing == false, "it returned only once the queue drained")
        try await waitUntil { try await lastRefreshed(rig.db, "b") ?? 0 > longAgo }
        #expect(try await lastRefreshed(rig.db, "a") ?? 0 > longAgo)
    }

    @Test(.timeLimit(.minutes(1)))
    func theBackgroundTaskReschedulesBeforeItRefreshes() async throws {
        let rig = try await makeRig { db, server in
            try await db.execute(.upsertSubscription(dueSubscription(server, id: "a")))
            server.reply("/a.rss", .init(status: 304, hold: true))
        }

        let task = Task { @MainActor in await rig.core.runBackgroundRefresh() }
        try await waitUntil { rig.server.requestCount(for: "/a.rss") == 1 } // mid-fetch

        #expect(rig.scheduled.value == 1, "the next wake-up is requested before the work, so a run cut short still chains")

        rig.server.release("/a.rss")
        await task.value
    }

    @Test(.timeLimit(.minutes(1)))
    func cancellingTheBackgroundTaskStopsFurtherFeedsButLetsTheActiveOneFinish() async throws {
        let rig = try await makeRig { db, server in
            try await db.execute(.upsertSubscription(dueSubscription(server, id: "a", title: "Alpha")))
            try await db.execute(.upsertSubscription(dueSubscription(server, id: "b", title: "Bravo")))
            server.reply("/a.rss", .init(status: 304, hold: true)) // frozen mid-flight
            server.reply("/b.rss", notModified())
        }
        let task = Task { @MainActor in await rig.core.refreshStaleAndWait() }
        try await waitUntil { rig.server.requestCount(for: "/a.rss") == 1 }

        task.cancel() // the system expires the task while feed a is still being fetched
        await task.value
        try await Task.sleep(for: .milliseconds(50)) // let the cancellation handler's event reach the core
        rig.server.release("/a.rss") // the in-flight fetch now completes
        await waitForIdle(rig.core)

        #expect(rig.server.requestedPaths == ["/a.rss"], "b must not start after the task was cancelled")
        try await waitUntil { try await lastRefreshed(rig.db, "a") ?? 0 > longAgo }
        #expect(try await lastRefreshed(rig.db, "b") == longAgo, "b is left for a later wake-up")
    }

    @Test(.timeLimit(.minutes(1)))
    func withoutCancellationBothFeedsAreFetched() async throws {
        // The control for the test above: the same setup, never cancelled, fetches b too.
        let rig = try await makeRig { db, server in
            try await db.execute(.upsertSubscription(dueSubscription(server, id: "a", title: "Alpha")))
            try await db.execute(.upsertSubscription(dueSubscription(server, id: "b", title: "Bravo")))
            server.reply("/a.rss", .init(status: 304, hold: true))
            server.reply("/b.rss", notModified())
        }
        let task = Task { @MainActor in await rig.core.refreshStaleAndWait() }
        try await waitUntil { rig.server.requestCount(for: "/a.rss") == 1 }

        rig.server.release("/a.rss")
        await task.value

        #expect(rig.server.requestedPaths == ["/a.rss", "/b.rss"])
    }

    // MARK: Downloads

    @Test(.timeLimit(.minutes(1)))
    func aBackgroundLaunchDoesNotResumeInterruptedDownloadsButActivationDoes() async throws {
        let rig = try await makeRig { db, server in
            var sub = dueSubscription(server, id: "s")
            sub.lastRefreshed = now() // not due: this test is about downloads, not refresh
            try await db.execute(.upsertSubscription(sub))
            try await db.execute(.upsertEpisode(Episode(
                id: "e1", feedGuid: "g1", subscriptionId: "s", title: "Interrupted", description: nil,
                pubDate: 1, durationSecs: 60, enclosureUrl: server.url("/e1.mp3"), artworkUrl: nil,
                playbackStatus: .unplayed, playbackPositionSecs: nil, downloadStatus: .downloading,
                isFlagged: false, fileSizeBytes: nil, localPath: nil,
            )))
            server.reply("/e1.mp3", .init(body: Data(repeating: 7, count: 2_048)))
        }

        // What the refresh task does: launch the core and refresh. It never becomes active.
        await rig.core.runBackgroundRefresh()
        try await Task.sleep(for: .milliseconds(200)) // generous: a stray resume would have fired by now
        #expect(rig.server.requestCount(for: "/e1.mp3") == 0, "a background launch must stay metadata-only")

        // The first foreground activation resumes it.
        rig.core.appBecameActive()
        try await waitUntil {
            guard case let .episode(episode) = try await rig.db.execute(.getEpisode(id: "e1")) else { return false }
            return episode.downloadStatus == .downloaded
        }
        #expect(rig.server.requestCount(for: "/e1.mp3") == 1)
    }
}

// MARK: - Playback is created on demand

/// Saves an episode (with an unusable, empty enclosure URL, so a play attempt fails
/// before any audio is touched) as the active play context.
private func seedPlayContext(_ db: DatabaseManager) async throws {
    let sub = Subscription(
        id: "sub", feedUrl: "https://example.com/sub.rss", title: "Feed", artworkUrl: nil,
        description: nil, lastRefreshed: longAgo, createdAt: longAgo, etag: nil, lastModified: nil,
        lastRefreshError: nil, retryAfterUntil: nil,
    )
    try await db.execute(.upsertSubscription(sub))
    let episode = Episode(
        id: "ep", feedGuid: "ep-guid", subscriptionId: "sub", title: "Episode", description: nil,
        pubDate: 1, durationSecs: 600, enclosureUrl: "", artworkUrl: nil,
        playbackStatus: .inProgress, playbackPositionSecs: 30, downloadStatus: .notDownloaded,
        isFlagged: false, fileSizeBytes: nil, localPath: nil,
    )
    try await db.execute(.upsertEpisode(episode))
    try await db.execute(.savePlayContext(episodeId: "ep", source: .subscription(id: "sub")))
}

@Test @MainActor func restoringTheSavedEpisodeDoesNotCreateTheAudioEngine() async throws {
    let rig = try await makeRig { db, _ in try await seedPlayContext(db) }

    // The mini-player comes back (this is also what a background-refresh launch does)…
    try await waitUntil { rig.core.view.player != nil }

    // …without an audio engine or lock-screen integration behind it.
    #expect(rig.core.view.player?.isPlaying == false)
    #expect(!rig.core.hasPlaybackEngine)
}

@Test @MainActor func theFirstPlayRequestCreatesTheAudioEngine() async throws {
    let rig = try await makeRig { db, _ in try await seedPlayContext(db) }
    try await waitUntil { rig.core.view.player != nil }
    #expect(!rig.core.hasPlaybackEngine)

    rig.core.update(.togglePlay)

    #expect(rig.core.hasPlaybackEngine)
}
