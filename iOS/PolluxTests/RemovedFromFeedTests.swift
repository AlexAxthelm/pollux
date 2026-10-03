import App
import Foundation
import Testing

@testable import Pollux

// MARK: - Helpers

/// A wall clock the tests move by hand. The removed-from-feed rule is about elapsed time,
/// so every test drives `DatabaseManager`'s injected clock rather than sleeping.
private final class TestClock: @unchecked Sendable {
    private let lock = NSLock()
    private var current = Date(timeIntervalSince1970: 1_800_000_000)

    var date: Date {
        lock.lock()
        defer { lock.unlock() }
        return current
    }

    func advance(hours: Double = 0, seconds: Double = 0) {
        lock.lock()
        defer { lock.unlock() }
        current = current.addingTimeInterval(hours * 3600 + seconds)
    }

    func set(_ date: Date) {
        lock.lock()
        defer { lock.unlock() }
        current = date
    }
}

private let grace = 48.0 // hours: the removal grace period

private func makeManager(clock: TestClock) throws -> DatabaseManager {
    let path = FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString + ".sqlite").path
    return try DatabaseManager(path: path, now: { clock.date })
}

/// The feed URL is derived from the id, so a subscription is the same row wherever a test
/// builds it (the upsert matches on feed URL, so they must agree).
private func makeSubscription(id: String = "sub-1") -> Subscription {
    Subscription(
        id: id, feedUrl: "https://example.com/\(id).rss", title: "Podcast", artworkUrl: nil, description: nil,
        lastRefreshed: 1_000, createdAt: 1_000, etag: nil, lastModified: nil,
        lastRefreshError: nil, retryAfterUntil: nil,
    )
}

private func makeEpisode(
    id: String, guid: String, subscriptionId: String = "sub-1",
    downloadStatus: DownloadStatus = .notDownloaded,
) -> Episode {
    Episode(
        id: id, feedGuid: guid, subscriptionId: subscriptionId, title: "Episode \(guid)",
        description: nil, pubDate: 1_000_000, durationSecs: 60,
        enclosureUrl: "https://example.com/\(guid).mp3", artworkUrl: nil,
        playbackStatus: .unplayed, playbackPositionSecs: nil, downloadStatus: downloadStatus,
        isFlagged: false, fileSizeBytes: nil, localPath: nil,
    )
}

/// One refresh whose feed contains exactly the episodes with these guids.
private func refresh(_ db: DatabaseManager, feedGuids: [String], subscriptionId: String = "sub-1") async throws {
    try await db.execute(.upsertFeedWithEpisodes(
        subscription: makeSubscription(id: subscriptionId),
        episodes: feedGuids.map { makeEpisode(id: "x-\($0)", guid: $0, subscriptionId: subscriptionId) },
    ))
}

private func statuses(_ db: DatabaseManager, subscriptionId: String = "sub-1") async throws -> [String: DownloadStatus] {
    let result = try await db.execute(.listEpisodesBySubscription(subscriptionId: subscriptionId))
    guard case let .episodes(rows) = result else {
        Issue.record("Expected .episodes, got \(result)")
        return [:]
    }
    return Dictionary(uniqueKeysWithValues: rows.map { ($0.feedGuid, $0.downloadStatus) })
}

/// A 304: nothing new from the server, only the outcome recorded. Returns how many
/// episodes that write flagged.
@discardableResult
private func unchangedRefresh(_ db: DatabaseManager, now: Int64) async throws -> UInt64? {
    let result = try await db.execute(.updateRefreshState(
        subscriptionId: "sub-1", lastRefreshed: now, lastRefreshError: nil, retryAfterUntil: nil,
    ))
    guard case let .episodesRemoved(count) = result else {
        Issue.record("Expected .episodesRemoved, got \(result)")
        return nil
    }
    return count
}

/// A manager with `sub-1` and episodes `keep` and `gone` stored, on a fresh clock.
private func seeded(_ guids: [String] = ["keep", "gone"]) async throws -> (DatabaseManager, TestClock) {
    let clock = TestClock()
    let db = try makeManager(clock: clock)
    try await db.execute(.upsertSubscription(makeSubscription()))
    for guid in guids {
        try await db.execute(.upsertEpisode(makeEpisode(id: guid, guid: guid)))
    }
    return (db, clock)
}

// MARK: - Tests

@Suite("Removed from feed")
struct RemovedFromFeedTests {
    @Test func theFirstMissNeverFlagsAnEpisode() async throws {
        // One truncated or stale response must not mark most of a feed removed.
        let (db, _) = try await seeded()

        try await refresh(db, feedGuids: ["keep"])

        #expect(try await statuses(db)["gone"] == .notDownloaded)
    }

    @Test func manyRefreshesInsideTheGraceWindowDoNotFlag() async throws {
        // The scenario behind the time-based rule: a stale CDN cache serves the same bad
        // feed for a while and the user pulls to refresh again and again. A refresh
        // *count* would be reached in minutes; elapsed time is not.
        let (db, clock) = try await seeded()

        try await refresh(db, feedGuids: ["keep"])
        for _ in 0 ..< 10 {
            clock.advance(hours: 4)
            try await refresh(db, feedGuids: ["keep"])
        }
        // 40 hours after the first miss, ten refreshes later.

        #expect(try await statuses(db)["gone"] == .notDownloaded)
    }

    @Test func anEpisodeStillMissingAfterTheGraceWindowIsFlagged() async throws {
        let (db, clock) = try await seeded()
        try await refresh(db, feedGuids: ["keep"])

        clock.advance(hours: grace)
        try await refresh(db, feedGuids: ["keep"])

        let rows = try await statuses(db)
        #expect(rows["gone"] == .removedFromFeed)
        #expect(rows["keep"] == .notDownloaded)
        #expect(rows.count == 2, "rows are kept, only flagged")
    }

    @Test func anUnchangedFeedFlagsAnAbsenceThatOutlastedTheGracePeriod() async throws {
        // A body omits `gone` once, then the feed stops changing and every refresh is a
        // 304. Nothing but the 304 can ever finish the job.
        let (db, clock) = try await seeded()
        try await refresh(db, feedGuids: ["keep"])

        clock.advance(hours: grace)
        let flagged = try await unchangedRefresh(db, now: 2_000)

        #expect(flagged == 1)
        let rows = try await statuses(db)
        #expect(rows["gone"] == .removedFromFeed)
        #expect(rows["keep"] == .notDownloaded)
    }

    @Test func anUnchangedFeedInsideTheGraceWindowFlagsNothing() async throws {
        let (db, clock) = try await seeded()
        try await refresh(db, feedGuids: ["keep"])

        clock.advance(hours: grace - 1)
        let flagged = try await unchangedRefresh(db, now: 2_000)

        #expect(flagged == 0)
        #expect(try await statuses(db)["gone"] == .notDownloaded)
    }

    @Test func anUnchangedFeedNeverStartsAClock() async throws {
        // No body was compared, so no episode is known to be missing. Had the 304 started
        // clocks, the second one (a full grace period later) would flag everything.
        let (db, clock) = try await seeded()

        try await unchangedRefresh(db, now: 2_000)
        clock.advance(hours: grace)
        let flagged = try await unchangedRefresh(db, now: 3_000)

        #expect(flagged == 0)
        let rows = try await statuses(db)
        #expect(rows["keep"] == .notDownloaded)
        #expect(rows["gone"] == .notDownloaded)
    }

    @Test func aFailedRefreshDoesNotConfirmAbsences() async throws {
        // Only a 304 says the stored list still matches the server. A 500 or a 429 says
        // nothing, so it must not flag even if the grace period has passed.
        let (db, clock) = try await seeded()
        try await refresh(db, feedGuids: ["keep"])

        clock.advance(hours: grace)
        let result = try await db.execute(.updateRefreshState(
            subscriptionId: "sub-1", lastRefreshed: nil, lastRefreshError: "HTTP 500", retryAfterUntil: 9_999,
        ))

        guard case let .episodesRemoved(count) = result else {
            Issue.record("Expected .episodesRemoved, got \(result)")
            return
        }
        #expect(count == 0)
        #expect(try await statuses(db)["gone"] == .notDownloaded)
    }

    @Test func theBoundaryIsExactlyTheGracePeriod() async throws {
        let (db, clock) = try await seeded()
        try await refresh(db, feedGuids: ["keep"])

        clock.advance(hours: grace, seconds: -1)
        try await refresh(db, feedGuids: ["keep"])
        #expect(try await statuses(db)["gone"] == .notDownloaded, "one second short")

        clock.advance(seconds: 1)
        try await refresh(db, feedGuids: ["keep"])
        #expect(try await statuses(db)["gone"] == .removedFromFeed, "exactly the grace period")
    }

    @Test func failedEpisodesAreFlaggedToo() async throws {
        let clock = TestClock()
        let db = try makeManager(clock: clock)
        try await db.execute(.upsertSubscription(makeSubscription()))
        try await db.execute(.upsertEpisode(makeEpisode(id: "keep", guid: "keep")))
        try await db.execute(.upsertEpisode(makeEpisode(id: "f", guid: "f", downloadStatus: .failed)))
        try await refresh(db, feedGuids: ["keep"])

        clock.advance(hours: grace)
        try await refresh(db, feedGuids: ["keep"])

        #expect(try await statuses(db)["f"] == .removedFromFeed)
    }

    @Test func aTruncatedResponseFollowedByTheFullFeedFlagsNothing() async throws {
        let all = ["a", "b", "c", "d", "e"]
        let (db, clock) = try await seeded(all)

        try await refresh(db, feedGuids: ["a"]) // truncated
        clock.advance(hours: 1)
        try await refresh(db, feedGuids: all) // recovered
        clock.advance(hours: grace * 2)
        try await refresh(db, feedGuids: all)

        #expect(try await statuses(db).values.allSatisfy { $0 == .notDownloaded })
    }

    @Test func reappearingRestartsTheClock() async throws {
        // Missing, present, missing is two separate absences, not one long one.
        let (db, clock) = try await seeded(["keep", "flaky"])

        try await refresh(db, feedGuids: ["keep"]) // first miss at t = 0
        clock.advance(hours: 24)
        try await refresh(db, feedGuids: ["keep", "flaky"]) // back: clock cleared
        clock.advance(hours: 6)
        try await refresh(db, feedGuids: ["keep"]) // missing again at t = 30h: new clock

        clock.advance(hours: grace - 1) // t = 77h: 47h after the new clock started
        try await refresh(db, feedGuids: ["keep"])
        #expect(try await statuses(db)["flaky"] == .notDownloaded, "the old clock must not count")

        clock.advance(hours: 1)
        try await refresh(db, feedGuids: ["keep"])
        #expect(try await statuses(db)["flaky"] == .removedFromFeed)
    }

    @Test func aFeedThatOnlyShowsItsLatestEpisodeStillFlagsTheOlderOnes() async throws {
        // A "latest only" feed really has dropped everything else; the grace period only
        // delays flagging those, it must not prevent it.
        let (db, clock) = try await seeded(["old-1", "old-2"])
        try await refresh(db, feedGuids: ["newest"])

        clock.advance(hours: grace)
        try await refresh(db, feedGuids: ["newest"])

        let rows = try await statuses(db)
        #expect(rows["old-1"] == .removedFromFeed)
        #expect(rows["old-2"] == .removedFromFeed)
        #expect(rows["newest"] == .notDownloaded)
    }

    @Test func downloadedAndInFlightEpisodesAreSpared() async throws {
        let clock = TestClock()
        let db = try makeManager(clock: clock)
        try await db.execute(.upsertSubscription(makeSubscription()))
        for (id, status) in [("done", DownloadStatus.downloaded), ("queued", .queued), ("active", .downloading)] {
            try await db.execute(.upsertEpisode(makeEpisode(id: id, guid: id, downloadStatus: status)))
        }
        try await refresh(db, feedGuids: ["other"])

        clock.advance(hours: grace * 3)
        try await refresh(db, feedGuids: ["other"])

        let rows = try await statuses(db)
        #expect(rows["done"] == .downloaded)
        #expect(rows["queued"] == .queued)
        #expect(rows["active"] == .downloading)
    }

    @Test func aDeletedDownloadOfALongMissingEpisodeIsFlaggedOnTheNextRefresh() async throws {
        // The clock keeps running for episodes that are exempt from flagging, so once the
        // file is deleted and the row is eligible again, the next refresh flags it.
        let clock = TestClock()
        let db = try makeManager(clock: clock)
        try await db.execute(.upsertSubscription(makeSubscription()))
        try await db.execute(.upsertEpisode(makeEpisode(id: "e1", guid: "e1", downloadStatus: .downloaded)))
        try await refresh(db, feedGuids: ["other"])
        clock.advance(hours: grace)
        try await refresh(db, feedGuids: ["other"])
        try await db.execute(.updateDownloadState(
            episodeId: "e1", status: .notDownloaded, localPath: nil, sizeBytes: nil,
        ))

        clock.advance(hours: 1)
        try await refresh(db, feedGuids: ["other"])

        #expect(try await statuses(db)["e1"] == .removedFromFeed)
    }

    @Test func anEmptyResponseNeitherFlagsNorStartsTheClock() async throws {
        let (db, clock) = try await seeded()

        try await db.execute(.upsertFeedWithEpisodes(subscription: makeSubscription(), episodes: []))
        clock.advance(hours: grace - 1)
        // First real miss for `gone`, 47h after the empty response. If the empty response
        // had started its clock, the next refresh would already be past the grace period.
        try await refresh(db, feedGuids: ["keep"])
        clock.advance(hours: 1)
        try await refresh(db, feedGuids: ["keep"])

        #expect(try await statuses(db)["gone"] == .notDownloaded, "only 1h since the first real miss")
    }

    @Test func aClockStartedInTheFutureIsRestartedNotTrusted() async throws {
        // The wall clock was ahead when the miss was recorded and has since been
        // corrected. The stale future timestamp must be replaced, not honoured.
        let (db, clock) = try await seeded()
        let start = clock.date
        clock.set(start.addingTimeInterval(365 * 24 * 3600)) // clock a year fast
        try await refresh(db, feedGuids: ["keep"]) // first miss recorded at +1y
        clock.set(start) // corrected

        try await refresh(db, feedGuids: ["keep"]) // clock restarts at the true time
        clock.advance(hours: grace)
        try await refresh(db, feedGuids: ["keep"])

        #expect(try await statuses(db)["gone"] == .removedFromFeed)
    }

    @Test func reappearingBecomesAvailableAgainAndClearsTheClock() async throws {
        let (db, clock) = try await seeded()
        try await refresh(db, feedGuids: ["keep"])
        clock.advance(hours: grace)
        try await refresh(db, feedGuids: ["keep"])
        #expect(try await statuses(db)["gone"] == .removedFromFeed)

        try await refresh(db, feedGuids: ["keep", "gone"])
        #expect(try await statuses(db)["gone"] == .notDownloaded)

        // Gone again: it needs a full new grace period, not the old clock.
        clock.advance(hours: 1)
        try await refresh(db, feedGuids: ["keep"])
        clock.advance(hours: grace - 1)
        try await refresh(db, feedGuids: ["keep"])
        #expect(try await statuses(db)["gone"] == .notDownloaded)
    }

    @Test func clocksAreScopedToTheirOwnSubscription() async throws {
        // Reconciling one feed must not touch another feed's episodes.
        let (db, clock) = try await seeded()
        try await db.execute(.upsertSubscription(makeSubscription(id: "sub-2")))
        try await db.execute(.upsertEpisode(makeEpisode(id: "theirs", guid: "theirs", subscriptionId: "sub-2")))

        try await refresh(db, feedGuids: ["keep"])
        clock.advance(hours: grace)
        try await refresh(db, feedGuids: ["keep"])

        #expect(try await statuses(db, subscriptionId: "sub-2")["theirs"] == .notDownloaded)
    }
}
