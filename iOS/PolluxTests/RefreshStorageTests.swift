import App
import Foundation
import Testing

@testable import Pollux

// MARK: - Helpers

private func makeManager() throws -> DatabaseManager {
    let path = FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString + ".sqlite").path
    return try DatabaseManager(path: path)
}

private let feedUrl = "https://example.com/feed.rss"

private func makeSubscription(
    id: String = "sub-1",
    etag: String? = nil,
    lastModified: String? = nil
) -> Subscription {
    Subscription(
        id: id,
        feedUrl: feedUrl,
        title: "Podcast",
        artworkUrl: nil,
        description: nil,
        lastRefreshed: 1_000,
        createdAt: 1_000,
        etag: etag,
        lastModified: lastModified,
        lastRefreshError: nil,
        retryAfterUntil: nil
    )
}

private func makeEpisode(
    id: String,
    guid: String,
    downloadStatus: DownloadStatus = .notDownloaded,
    fileSizeBytes: UInt64? = nil
) -> Episode {
    Episode(
        id: id,
        feedGuid: guid,
        subscriptionId: "sub-1",
        title: "Episode \(guid)",
        description: nil,
        pubDate: 1_000_000,
        durationSecs: 60,
        enclosureUrl: "https://example.com/\(guid).mp3",
        artworkUrl: nil,
        playbackStatus: .unplayed,
        playbackPositionSecs: nil,
        downloadStatus: downloadStatus,
        isFlagged: false,
        fileSizeBytes: fileSizeBytes,
        localPath: nil
    )
}

private func episodes(_ db: DatabaseManager) async throws -> [String: Episode] {
    let result = try await db.execute(.listEpisodesBySubscription(subscriptionId: "sub-1"))
    guard case let .episodes(rows) = result else {
        Issue.record("Expected .episodes, got \(result)")
        return [:]
    }
    return Dictionary(uniqueKeysWithValues: rows.map { ($0.feedGuid, $0) })
}

private func subscription(_ db: DatabaseManager) async throws -> Subscription? {
    guard case let .subscription(sub) = try await db.execute(.getSubscription(id: "sub-1")) else {
        return nil
    }
    return sub
}

// MARK: - Tests

@Suite("Refresh storage")
struct RefreshStorageTests {
    @Test func validatorsRoundTripThroughUpsert() async throws {
        let db = try makeManager()
        let saved = try await db.execute(.upsertFeedWithEpisodes(
            subscription: makeSubscription(etag: "\"v1\"", lastModified: "Wed, 01 Oct 2026 00:00:00 GMT"),
            episodes: [],
        ))
        guard case let .subscription(sub) = saved else {
            Issue.record("Expected .subscription, got \(saved)")
            return
        }
        #expect(sub.etag == "\"v1\"")
        #expect(sub.lastModified == "Wed, 01 Oct 2026 00:00:00 GMT")
    }

    @Test func updateRefreshState_recordsFailureAndKeepsTimestamp() async throws {
        let db = try makeManager()
        try await db.execute(.upsertSubscription(makeSubscription(etag: "\"v1\"")))

        try await db.execute(.updateRefreshState(
            subscriptionId: "sub-1", lastRefreshed: nil,
            lastRefreshError: "HTTP 500", retryAfterUntil: 9_999,
        ))

        let sub = try #require(try await subscription(db))
        #expect(sub.lastRefreshError == "HTTP 500")
        #expect(sub.retryAfterUntil == 9_999)
        #expect(sub.lastRefreshed == 1_000, "a failure must not move last_refreshed")
        #expect(sub.etag == "\"v1\"", "validators survive a failed refresh")
    }

    @Test func updateRefreshState_successClearsFailure() async throws {
        let db = try makeManager()
        try await db.execute(.upsertSubscription(makeSubscription()))
        try await db.execute(.updateRefreshState(
            subscriptionId: "sub-1", lastRefreshed: nil,
            lastRefreshError: "boom", retryAfterUntil: 5,
        ))

        try await db.execute(.updateRefreshState(
            subscriptionId: "sub-1", lastRefreshed: 2_000,
            lastRefreshError: nil, retryAfterUntil: nil,
        ))

        let sub = try #require(try await subscription(db))
        #expect(sub.lastRefreshed == 2_000)
        #expect(sub.lastRefreshError == nil)
        #expect(sub.retryAfterUntil == nil)
    }

    @Test func refreshKeepsRealSizeOfDownloadedEpisode() async throws {
        let db = try makeManager()
        try await db.execute(.upsertSubscription(makeSubscription()))
        try await db.execute(.upsertEpisode(makeEpisode(id: "e1", guid: "g1", fileSizeBytes: 1_000)))
        try await db.execute(.updateDownloadState(
            episodeId: "e1", status: .downloaded, localPath: "e1.mp3", sizeBytes: 4_321,
        ))

        // The refreshed feed advertises a different <enclosure length>.
        try await db.execute(.upsertFeedWithEpisodes(
            subscription: makeSubscription(),
            episodes: [makeEpisode(id: "x", guid: "g1", fileSizeBytes: 9_999)],
        ))

        let ep = try #require(try await episodes(db)["g1"])
        #expect(ep.fileSizeBytes == 4_321, "on-disk size wins over the feed's advertised length")
        #expect(ep.downloadStatus == .downloaded)
        #expect(ep.localPath == "e1.mp3")
    }

    @Test func refreshStillUpdatesAdvertisedSizeWhenNotDownloaded() async throws {
        let db = try makeManager()
        try await db.execute(.upsertSubscription(makeSubscription()))
        try await db.execute(.upsertEpisode(makeEpisode(id: "e1", guid: "g1", fileSizeBytes: 1_000)))

        try await db.execute(.upsertFeedWithEpisodes(
            subscription: makeSubscription(),
            episodes: [makeEpisode(id: "x", guid: "g1", fileSizeBytes: 2_000)],
        ))

        #expect(try await episodes(db)["g1"]?.fileSizeBytes == 2_000)
    }

    @Test func reappearingEpisodeBecomesAvailableAgain() async throws {
        let db = try makeManager()
        try await db.execute(.upsertSubscription(makeSubscription()))
        try await db.execute(.upsertEpisode(makeEpisode(id: "e1", guid: "g1", downloadStatus: .removedFromFeed)))

        try await db.execute(.upsertFeedWithEpisodes(
            subscription: makeSubscription(),
            episodes: [makeEpisode(id: "x", guid: "g1")],
        ))

        #expect(try await episodes(db)["g1"]?.downloadStatus == .notDownloaded)
    }

    @Test func successfulUpsertClearsPriorFailure() async throws {
        let db = try makeManager()
        try await db.execute(.upsertSubscription(makeSubscription()))
        try await db.execute(.updateRefreshState(
            subscriptionId: "sub-1", lastRefreshed: nil,
            lastRefreshError: "boom", retryAfterUntil: 5,
        ))

        try await db.execute(.upsertFeedWithEpisodes(subscription: makeSubscription(), episodes: []))

        let sub = try #require(try await subscription(db))
        #expect(sub.lastRefreshError == nil)
        #expect(sub.retryAfterUntil == nil)
    }
}
