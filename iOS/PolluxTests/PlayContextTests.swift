import App
import Foundation
import Testing

@testable import Pollux

private func makeManager() throws -> DatabaseManager {
    let path = FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString + ".sqlite").path
    return try DatabaseManager(path: path)
}

private func seed(_ db: DatabaseManager, episodeId: String) async throws {
    let sub = Subscription(
        id: "sub-1", feedUrl: "https://example.com/sub-1.rss", title: "Feed",
        artworkUrl: nil, description: nil, lastRefreshed: nil, createdAt: 1)
    try await db.execute(.upsertSubscription(sub))
    let episode = Episode(
        id: episodeId, feedGuid: "guid-\(episodeId)", subscriptionId: "sub-1",
        title: "Episode", description: nil, pubDate: 1, durationSecs: 600,
        enclosureUrl: "https://example.com/\(episodeId).mp3", artworkUrl: nil,
        playbackStatus: .inProgress, playbackPositionSecs: 120,
        downloadStatus: .notDownloaded, isFlagged: false, fileSizeBytes: nil, localPath: nil)
    try await db.execute(.upsertEpisode(episode))
}

@Suite struct PlayContextTests {
    @Test func nothingSavedIsNotFound() async throws {
        let db = try makeManager()
        let result = try await db.execute(.loadPlayContext)
        guard case .notFound = result else {
            Issue.record("Expected .notFound, got \(result)")
            return
        }
    }

    @Test func savedContextLoadsTheEpisodeWithItsPosition() async throws {
        let db = try makeManager()
        try await seed(db, episodeId: "ep-1")
        try await db.execute(.savePlayContext(episodeId: "ep-1"))

        let result = try await db.execute(.loadPlayContext)
        guard case let .episode(episode) = result else {
            Issue.record("Expected .episode, got \(result)")
            return
        }
        #expect(episode.id == "ep-1")
        #expect(episode.playbackPositionSecs == 120)
    }

    @Test func savingReplacesThePreviousContext() async throws {
        let db = try makeManager()
        try await seed(db, episodeId: "ep-1")
        try await seed(db, episodeId: "ep-2")
        try await db.execute(.savePlayContext(episodeId: "ep-1"))
        try await db.execute(.savePlayContext(episodeId: "ep-2"))

        guard case let .episode(episode) = try await db.execute(.loadPlayContext) else {
            Issue.record("Expected an episode")
            return
        }
        #expect(episode.id == "ep-2")
    }

    @Test func clearingRemovesIt() async throws {
        let db = try makeManager()
        try await seed(db, episodeId: "ep-1")
        try await db.execute(.savePlayContext(episodeId: "ep-1"))
        try await db.execute(.clearPlayContext)

        guard case .notFound = try await db.execute(.loadPlayContext) else {
            Issue.record("Expected .notFound after clearing")
            return
        }
    }

    @Test func deletingTheSubscriptionDropsTheContext() async throws {
        let db = try makeManager()
        try await seed(db, episodeId: "ep-1")
        try await db.execute(.savePlayContext(episodeId: "ep-1"))
        try await db.execute(.deleteSubscription(id: "sub-1"))

        guard case .notFound = try await db.execute(.loadPlayContext) else {
            Issue.record("Expected .notFound after the episode was deleted")
            return
        }
    }
}
