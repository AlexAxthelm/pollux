import App
import Foundation
import GRDB
import Testing

@testable import Pollux

// MARK: - Helpers

private func makeManager() throws -> DatabaseManager {
    let path = FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString + ".sqlite").path
    return try DatabaseManager(path: path)
}

private func makeSubscription(
    id: String = UUID().uuidString,
    title: String = "Test Podcast",
    feedUrl: String? = nil
) -> Subscription {
    Subscription(
        id: id,
        feedUrl: feedUrl ?? "https://example.com/\(id).rss",
        title: title,
        artworkUrl: nil,
        description: nil,
        lastRefreshed: nil,
        createdAt: 1_000_000
    )
}

private func makeEpisode(
    id: String = UUID().uuidString,
    subscriptionId: String,
    feedGuid: String? = nil,
    title: String = "Test Episode",
    playbackStatus: PlaybackStatus = .unplayed,
    downloadStatus: DownloadStatus = .notDownloaded,
    fileSizeBytes: UInt64? = nil
) -> Episode {
    Episode(
        id: id,
        feedGuid: feedGuid ?? "guid-\(id)",
        subscriptionId: subscriptionId,
        title: title,
        description: nil,
        pubDate: 1_000_000,
        durationSecs: 3600,
        enclosureUrl: "https://example.com/\(id).mp3",
        artworkUrl: nil,
        playbackStatus: playbackStatus,
        playbackPositionSecs: nil,
        downloadStatus: downloadStatus,
        downloadProgress: nil,
        isFlagged: false,
        fileSizeBytes: fileSizeBytes,
        localPath: nil
    )
}

// MARK: - Tests

@Suite("DatabaseManager")
struct DatabaseManagerTests {

    @Test func migrationsRunWithoutError() throws {
        _ = try makeManager()
    }

    // MARK: Subscriptions

    @Test func listSubscriptions_emptyOnFreshDatabase() async throws {
        let db = try makeManager()
        let result = try await db.execute(.listSubscriptions)
        guard case .subscriptions(let subs) = result else {
            Issue.record("Expected .subscriptions, got \(result)")
            return
        }
        #expect(subs.isEmpty)
    }

    @Test func upsertAndRetrieveSubscription() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1", title: "Test Podcast Title")

        let upsertResult = try await db.execute(.upsertSubscription(sub))
        #expect(upsertResult == .success)

        let getResult = try await db.execute(.getSubscription(id: "sub-1"))
        guard case .subscription(let fetched) = getResult else {
            Issue.record("Expected .subscription, got \(getResult)")
            return
        }
        #expect(fetched.id == "sub-1")
        #expect(fetched.title == "Test Podcast Title")
        #expect(fetched.feedUrl == sub.feedUrl)
    }

    @Test func upsertSubscription_updatesOnConflict() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1", title: "Title Before Update")
        try await db.execute(.upsertSubscription(sub))

        let updated = Subscription(
            id: "sub-1",
            feedUrl: sub.feedUrl,
            title: "Title After Update",
            artworkUrl: "https://example.com/art.png",
            description: sub.description,
            lastRefreshed: 2_000_000,
            createdAt: sub.createdAt
        )
        try await db.execute(.upsertSubscription(updated))

        let result = try await db.execute(.getSubscription(id: "sub-1"))
        guard case .subscription(let fetched) = result else { return }
        #expect(fetched.title == "Title After Update")
        #expect(fetched.artworkUrl == "https://example.com/art.png")
        #expect(fetched.lastRefreshed == 2_000_000)
    }

    @Test func deleteSubscription() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1")
        try await db.execute(.upsertSubscription(sub))

        try await db.execute(.deleteSubscription(id: "sub-1"))

        let result = try await db.execute(.getSubscription(id: "sub-1"))
        #expect(result == .notFound)
    }

    @Test func upsertSubscription_duplicateFeedUrl_updatesExisting() async throws {
        let db = try makeManager()
        let feedUrl = "https://example.com/test.rss"
        let original = makeSubscription(id: "sub-1", title: "Original Title", feedUrl: feedUrl)
        try await db.execute(.upsertSubscription(original))

        let duplicate = makeSubscription(id: "sub-2", title: "Updated Title", feedUrl: feedUrl)
        try await db.execute(.upsertSubscription(duplicate))

        let result = try await db.execute(.listSubscriptions)
        guard case .subscriptions(let subs) = result else { return }
        #expect(subs.count == 1, "same feed_url should upsert, not insert a duplicate")
        #expect(subs[0].title == "Updated Title")
    }

    @Test func listSubscriptions_sortedByTitle() async throws {
        let db = try makeManager()
        try await db.execute(.upsertSubscription(makeSubscription(id: "b", title: "zebra cast")))
        try await db.execute(.upsertSubscription(makeSubscription(id: "a", title: "Apple Talks")))
        try await db.execute(.upsertSubscription(makeSubscription(id: "c", title: "middle ground")))

        let result = try await db.execute(.listSubscriptions)
        guard case .subscriptions(let subs) = result else { return }
        #expect(subs.map(\.title) == ["Apple Talks", "middle ground", "zebra cast"])
    }

    // MARK: Episodes

    @Test func upsertAndListEpisodes() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1")
        try await db.execute(.upsertSubscription(sub))

        let ep1 = makeEpisode(subscriptionId: "sub-1", title: "Episode 1")
        let ep2 = makeEpisode(subscriptionId: "sub-1", title: "Episode 2")
        try await db.execute(.upsertEpisode(ep1))
        try await db.execute(.upsertEpisode(ep2))

        let result = try await db.execute(.listEpisodesBySubscription(subscriptionId: "sub-1"))
        guard case .episodes(let eps) = result else {
            Issue.record("Expected .episodes, got \(result)")
            return
        }
        #expect(eps.count == 2)
    }

    @Test func getEpisodeByFeedGuid() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1")
        try await db.execute(.upsertSubscription(sub))

        let ep = makeEpisode(subscriptionId: "sub-1", feedGuid: "rss-guid-42", title: "Episode With Specific Guid")
        try await db.execute(.upsertEpisode(ep))

        let result = try await db.execute(
            .getEpisodeByFeedGuid(subscriptionId: "sub-1", feedGuid: "rss-guid-42"))
        guard case .episode(let fetched) = result else {
            Issue.record("Expected .episode, got \(result)")
            return
        }
        #expect(fetched.title == "Episode With Specific Guid")
        #expect(fetched.feedGuid == "rss-guid-42")
    }

    @Test func updatePlaybackStatus() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1")
        try await db.execute(.upsertSubscription(sub))

        let ep = makeEpisode(id: "ep-1", subscriptionId: "sub-1")
        try await db.execute(.upsertEpisode(ep))

        try await db.execute(
            .updatePlaybackStatus(
                episodeId: "ep-1", status: .inProgress, positionSecs: 42))

        let result = try await db.execute(.getEpisode(id: "ep-1"))
        guard case .episode(let fetched) = result else { return }
        #expect(fetched.playbackStatus == .inProgress)
        #expect(fetched.playbackPositionSecs == 42)
    }

    @Test func updatePlaybackStatus_largePositionRoundTrips() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1")
        try await db.execute(.upsertSubscription(sub))
        let ep = makeEpisode(id: "ep-1", subscriptionId: "sub-1")
        try await db.execute(.upsertEpisode(ep))

        // UInt32 values above Int32.max would produce negative values via Int32(bitPattern:),
        // violating the >= 0 constraint. Verify the full UInt32 range survives the round-trip.
        let largePosition: UInt32 = UInt32(Int32.max) + 1
        try await db.execute(
            .updatePlaybackStatus(episodeId: "ep-1", status: .inProgress, positionSecs: largePosition))

        let result = try await db.execute(.getEpisode(id: "ep-1"))
        guard case .episode(let fetched) = result else { return }
        #expect(fetched.playbackPositionSecs == largePosition)
    }

    @Test func deleteSubscription_cascadesToEpisodes() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1")
        try await db.execute(.upsertSubscription(sub))
        try await db.execute(.upsertEpisode(makeEpisode(subscriptionId: "sub-1")))
        try await db.execute(.upsertEpisode(makeEpisode(subscriptionId: "sub-1")))

        try await db.execute(.deleteSubscription(id: "sub-1"))

        let result = try await db.execute(.listEpisodesBySubscription(subscriptionId: "sub-1"))
        guard case .episodes(let eps) = result else { return }
        #expect(eps.isEmpty)
    }

    @Test func getSubscription_notFound() async throws {
        let db = try makeManager()
        let result = try await db.execute(.getSubscription(id: "nonexistent"))
        #expect(result == .notFound)
    }

    @Test func getEpisode_notFound() async throws {
        let db = try makeManager()
        let result = try await db.execute(.getEpisode(id: "nonexistent"))
        #expect(result == .notFound)
    }

    // MARK: Status round-trips

    @Test func playbackStatus_allCasesRoundTrip() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1")
        try await db.execute(.upsertSubscription(sub))

        let cases: [(String, PlaybackStatus)] = [
            ("ep-unplayed", .unplayed),
            ("ep-inprogress", .inProgress),
            ("ep-played", .played),
        ]
        for (id, status) in cases {
            try await db.execute(.upsertEpisode(makeEpisode(id: id, subscriptionId: "sub-1", playbackStatus: status)))
        }
        for (id, expected) in cases {
            let result = try await db.execute(.getEpisode(id: id))
            guard case .episode(let ep) = result else {
                Issue.record("Expected .episode for id \(id), got \(result)")
                continue
            }
            #expect(ep.playbackStatus == expected, "id: \(id)")
        }
    }

    @Test func downloadStatus_allCasesRoundTrip() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1")
        try await db.execute(.upsertSubscription(sub))

        let cases: [(String, DownloadStatus)] = [
            ("ep-notdownloaded", .notDownloaded),
            ("ep-queued", .queued),
            ("ep-downloading", .downloading),
            ("ep-downloaded", .downloaded),
            ("ep-failed", .failed),
            ("ep-removedfromfeed", .removedFromFeed),
        ]
        for (id, status) in cases {
            try await db.execute(.upsertEpisode(makeEpisode(id: id, subscriptionId: "sub-1", downloadStatus: status)))
        }
        for (id, expected) in cases {
            let result = try await db.execute(.getEpisode(id: id))
            guard case .episode(let ep) = result else {
                Issue.record("Expected .episode for id \(id), got \(result)")
                continue
            }
            #expect(ep.downloadStatus == expected, "id: \(id)")
        }
    }

    // MARK: UpsertFeedWithEpisodes

    @Test func upsertFeedWithEpisodes_createsNewSubscriptionAndEpisodes() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-new", title: "New Feed", feedUrl: "https://example.com/new.rss")
        let ep1 = makeEpisode(id: "ep-1", subscriptionId: "sub-new", feedGuid: "guid-1", title: "Episode One")
        let ep2 = makeEpisode(id: "ep-2", subscriptionId: "sub-new", feedGuid: "guid-2", title: "Episode Two")

        let result = try await db.execute(.upsertFeedWithEpisodes(subscription: sub, episodes: [ep1, ep2]))
        guard case .subscription(let saved) = result else {
            Issue.record("Expected .subscription, got \(result)")
            return
        }
        #expect(saved.title == "New Feed")
        #expect(saved.feedUrl == "https://example.com/new.rss")

        let epResult = try await db.execute(.listEpisodesBySubscription(subscriptionId: saved.id))
        guard case .episodes(let eps) = epResult else { return }
        #expect(eps.count == 2)
        #expect(eps.map(\.subscriptionId).allSatisfy { $0 == saved.id })
    }

    @Test func upsertFeedWithEpisodes_refreshUpdatesMetadataAndPreservesId() async throws {
        let db = try makeManager()
        let feedUrl = "https://example.com/existing.rss"
        let original = makeSubscription(id: "original-id", title: "Original Title", feedUrl: feedUrl)
        try await db.execute(.upsertSubscription(original))

        // Refresh: Rust sends a new UUID id but same feed_url
        let refreshed = makeSubscription(id: UUID().uuidString, title: "Updated Title", feedUrl: feedUrl)
        let ep = makeEpisode(id: "ep-1", subscriptionId: refreshed.id, feedGuid: "guid-1")

        let result = try await db.execute(.upsertFeedWithEpisodes(subscription: refreshed, episodes: [ep]))
        guard case .subscription(let saved) = result else {
            Issue.record("Expected .subscription, got \(result)")
            return
        }
        // Existing id must be preserved
        #expect(saved.id == "original-id", "should preserve existing subscription id on refresh")
        #expect(saved.title == "Updated Title")

        // Episode should be linked to canonical id
        let epResult = try await db.execute(.listEpisodesBySubscription(subscriptionId: "original-id"))
        guard case .episodes(let eps) = epResult else { return }
        #expect(eps.count == 1)
        #expect(eps[0].subscriptionId == "original-id")
    }

    @Test func upsertFeedWithEpisodes_updatesExistingEpisodeMetadata() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1", feedUrl: "https://example.com/feed.rss")
        try await db.execute(.upsertSubscription(sub))

        let ep = makeEpisode(id: "ep-1", subscriptionId: "sub-1", feedGuid: "stable-guid", title: "Old Title")
        try await db.execute(.upsertEpisode(ep))

        let updatedSub = makeSubscription(id: UUID().uuidString, feedUrl: "https://example.com/feed.rss")
        let updatedEp = makeEpisode(id: UUID().uuidString, subscriptionId: updatedSub.id, feedGuid: "stable-guid", title: "New Title")
        try await db.execute(.upsertFeedWithEpisodes(subscription: updatedSub, episodes: [updatedEp]))

        let result = try await db.execute(.getEpisodeByFeedGuid(subscriptionId: "sub-1", feedGuid: "stable-guid"))
        guard case .episode(let fetched) = result else { return }
        #expect(fetched.title == "New Title")
        // id should be preserved from original insert
        #expect(fetched.id == "ep-1")
    }

    @Test func upsertFeedWithEpisodes_refreshUpdatesFileSize() async throws {
        // file_size_bytes is feed metadata and can change when a publisher
        // re-encodes an episode, so a refresh must not leave it stale.
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1", feedUrl: "https://example.com/feed.rss")
        try await db.execute(.upsertSubscription(sub))

        let ep = makeEpisode(
            id: "ep-1", subscriptionId: "sub-1", feedGuid: "stable-guid", fileSizeBytes: 1000,
        )
        try await db.execute(.upsertEpisode(ep))

        let refreshed = makeEpisode(
            id: UUID().uuidString, subscriptionId: "sub-1", feedGuid: "stable-guid",
            fileSizeBytes: 2000,
        )
        try await db.execute(.upsertFeedWithEpisodes(subscription: sub, episodes: [refreshed]))

        let result = try await db.execute(
            .getEpisodeByFeedGuid(subscriptionId: "sub-1", feedGuid: "stable-guid"),
        )
        guard case let .episode(fetched) = result else {
            Issue.record("Expected .episode, got \(result)")
            return
        }
        #expect(fetched.fileSizeBytes == 2000)
        #expect(fetched.id == "ep-1", "refresh should preserve the original row")
    }

    @Test func upsertFeedWithEpisodes_noEpisodesIsValid() async throws {
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1", feedUrl: "https://example.com/empty.rss")

        let result = try await db.execute(.upsertFeedWithEpisodes(subscription: sub, episodes: []))
        guard case .subscription(let saved) = result else {
            Issue.record("Expected .subscription, got \(result)")
            return
        }
        #expect(saved.id == "sub-1")
    }

    // MARK: Schema constraints

    @Test func upsertEpisode_overflowFileSizeStoredAsNull() async throws {
        // UInt64 values > Int64.max can't be stored as a non-negative Int64.
        // The DB layer stores NULL rather than a negative value that would
        // violate the file_size_bytes >= 0 constraint.
        let db = try makeManager()
        let sub = makeSubscription(id: "sub-1")
        try await db.execute(.upsertSubscription(sub))

        let ep = Episode(
            id: "ep-1", feedGuid: "guid-1", subscriptionId: "sub-1",
            title: "Test", description: nil, pubDate: nil, durationSecs: nil,
            enclosureUrl: "https://example.com/ep.mp3", artworkUrl: nil,
            playbackStatus: .unplayed, playbackPositionSecs: nil,
            downloadStatus: .notDownloaded, downloadProgress: nil,
            isFlagged: false, fileSizeBytes: UInt64.max, localPath: nil
        )

        try await db.execute(.upsertEpisode(ep))

        let result = try await db.execute(.getEpisode(id: "ep-1"))
        guard case .episode(let fetched) = result else {
            Issue.record("Expected .episode, got \(result)")
            return
        }
        #expect(fetched.fileSizeBytes == nil, "overflow file size should be stored as NULL")
    }

    @Test func episode_outOfRangeDurationReadsAsNilNotCrash() async throws {
        // duration_secs is UInt32 in the typed API, so an out-of-range value can
        // only arrive from a later schema or external tooling. Plant one via raw
        // SQL and confirm the read degrades to nil instead of trapping.
        let path = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString + ".sqlite").path
        let db = try DatabaseManager(path: path)
        try await db.execute(.upsertSubscription(makeSubscription(id: "sub-1")))
        try await db.execute(.upsertEpisode(
            makeEpisode(id: "ep-1", subscriptionId: "sub-1", feedGuid: "g1"),
        ))

        let raw = try DatabasePool(path: path)
        try await raw.write { db in
            // 5_000_000_000 > UInt32.max (4_294_967_295)
            try db.execute(sql: "UPDATE episodes SET duration_secs = 5000000000 WHERE id = 'ep-1'")
        }

        let result = try await db.execute(.getEpisode(id: "ep-1"))
        guard case let .episode(fetched) = result else {
            Issue.record("Expected .episode, got \(result)")
            return
        }
        #expect(fetched.durationSecs == nil, "out-of-range duration should read as nil")
    }

    @Test func episode_outOfRangeDownloadProgressReadsAsNil() async throws {
        // download_progress has a CHECK (0...100), so planting an out-of-range
        // value requires bypassing it — standing in for a future schema that
        // drops the bound or external tooling that ignores it. The read must
        // degrade to nil, not silently wrap (300 -> 44) as truncation would.
        let path = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString + ".sqlite").path
        let db = try DatabaseManager(path: path)
        try await db.execute(.upsertSubscription(makeSubscription(id: "sub-1")))
        try await db.execute(.upsertEpisode(
            makeEpisode(id: "ep-1", subscriptionId: "sub-1", feedGuid: "g1"),
        ))

        let raw = try DatabasePool(path: path)
        try await raw.write { db in
            try db.execute(sql: "PRAGMA ignore_check_constraints = ON")
            try db.execute(sql: "UPDATE episodes SET download_progress = 300 WHERE id = 'ep-1'")
        }

        let result = try await db.execute(.getEpisode(id: "ep-1"))
        guard case let .episode(fetched) = result else {
            Issue.record("Expected .episode, got \(result)")
            return
        }
        #expect(fetched.downloadProgress == nil, "out-of-range progress should read as nil")
    }
}
