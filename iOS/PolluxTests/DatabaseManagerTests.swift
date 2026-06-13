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
    downloadStatus: DownloadStatus = .notDownloaded
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
        fileSizeBytes: nil,
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
}
