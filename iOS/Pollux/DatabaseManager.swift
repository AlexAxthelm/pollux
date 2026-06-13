import App
import Foundation
import GRDB

actor DatabaseManager {
    private let db: DatabasePool

    init() throws {
        let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first!
        try FileManager.default.createDirectory(at: support, withIntermediateDirectories: true)
        db = try DatabasePool(path: support.appendingPathComponent("pollux.sqlite").path)
        try Self.runMigrations(db)
    }

    /// Accepts an explicit path — use this in tests to point at a temp file.
    init(path: String) throws {
        db = try DatabasePool(path: path)
        try Self.runMigrations(db)
    }

    // MARK: - Migrations

    private static func runMigrations(_ db: DatabasePool) throws {
        var migrator = DatabaseMigrator()
        migrator.registerMigration("v1_initial") { db in
            try db.create(table: "subscriptions") { t in
                t.column("id", .text).primaryKey()
                t.column("feed_url", .text).notNull().unique()
                t.column("title", .text).notNull()
                t.column("artwork_url", .text)
                t.column("description", .text)
                t.column("last_refreshed", .integer)
                t.column("created_at", .integer).notNull()
            }
            try db.create(table: "episodes") { t in
                t.column("id", .text).primaryKey()
                t.column("feed_guid", .text).notNull()
                t.column("subscription_id", .text).notNull()
                    .references("subscriptions", column: "id", onDelete: .cascade)
                t.column("title", .text).notNull()
                t.column("description", .text)
                t.column("pub_date", .integer)
                t.column("duration_secs", .integer).check(sql: "duration_secs >= 0")
                t.column("enclosure_url", .text).notNull()
                t.column("artwork_url", .text)
                t.column("playback_status", .text).notNull().defaults(to: "Unplayed")
                t.column("playback_position_secs", .integer).check(sql: "playback_position_secs >= 0")
                t.column("download_status", .text).notNull().defaults(to: "NotDownloaded")
                t.column("download_progress", .integer).check(sql: "download_progress BETWEEN 0 AND 100")
                t.column("is_flagged", .boolean).notNull().defaults(to: false)
                t.column("file_size_bytes", .integer).check(sql: "file_size_bytes >= 0")
                t.column("local_path", .text)
                t.uniqueKey(["subscription_id", "feed_guid"])
            }
            try db.create(
                index: "episodes_subscription_id",
                on: "episodes",
                columns: ["subscription_id"],
            )
        }
        try migrator.migrate(db)
    }

    // MARK: - Execute

    @discardableResult
    func execute(_ operation: StorageOperation) async throws -> StorageResult {
        switch operation {
        case .listSubscriptions:
            try listSubscriptions()
        case let .getSubscription(id):
            try getSubscription(id: id)
        case let .upsertSubscription(sub):
            try await upsertSubscription(sub)
        case let .deleteSubscription(id):
            try await deleteSubscription(id: id)
        case let .upsertEpisode(episode):
            try await upsertEpisode(episode)
        case let .getEpisode(id):
            try getEpisode(id: id)
        case let .listEpisodesBySubscription(subscriptionId):
            try listEpisodesBySubscription(subscriptionId: subscriptionId)
        case let .getEpisodeByFeedGuid(subscriptionId, feedGuid):
            try getEpisodeByFeedGuid(subscriptionId: subscriptionId, feedGuid: feedGuid)
        case let .updatePlaybackStatus(episodeId, status, positionSecs):
            try await updatePlaybackStatus(episodeId: episodeId, status: status, positionSecs: positionSecs)
        }
    }

    // MARK: - Subscription operations

    private func listSubscriptions() throws -> StorageResult {
        let rows = try db.read { db -> [Row] in
            return try Row.fetchAll(db, sql: "SELECT * FROM subscriptions ORDER BY title COLLATE NOCASE")
        }
        return .subscriptions(rows.map(Self.subscription(from:)))
    }

    private func getSubscription(id: String) throws -> StorageResult {
        let row = try db.read { db -> Row? in
            return try Row.fetchOne(
                db, sql: "SELECT * FROM subscriptions WHERE id = ?", arguments: [id],
            )
        }
        guard let row else { return .notFound }
        return .subscription(Self.subscription(from: row))
    }

    private func upsertSubscription(_ sub: Subscription) async throws -> StorageResult {
        try await db.write { db in
            try db.execute(
                sql: """
                INSERT INTO subscriptions
                    (id, feed_url, title, artwork_url, description, last_refreshed, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title,
                    artwork_url = excluded.artwork_url,
                    description = excluded.description,
                    last_refreshed = excluded.last_refreshed
                """,
                arguments: [
                    sub.id, sub.feedUrl, sub.title, sub.artworkUrl,
                    sub.description, sub.lastRefreshed, sub.createdAt,
                ],
            )
        }
        return .success
    }

    private func deleteSubscription(id: String) async throws -> StorageResult {
        try await db.write { db in
            try db.execute(sql: "DELETE FROM subscriptions WHERE id = ?", arguments: [id])
        }
        return .success
    }

    // MARK: - Episode operations

    private func upsertEpisode(_ episode: Episode) async throws -> StorageResult {
        let playbackStr = Self.playbackStatusString(episode.playbackStatus)
        let downloadStr = Self.downloadStatusString(episode.downloadStatus)
        let downloadProgress = episode.downloadProgress.map { Int32($0) }
        try await db.write { db in
            try db.execute(
                sql: """
                INSERT INTO episodes
                    (id, feed_guid, subscription_id, title, description, pub_date,
                     duration_secs, enclosure_url, artwork_url, playback_status,
                     playback_position_secs, download_status, download_progress,
                     is_flagged, file_size_bytes, local_path)
                VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
                ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title,
                    description = excluded.description,
                    enclosure_url = excluded.enclosure_url,
                    artwork_url = excluded.artwork_url,
                    pub_date = excluded.pub_date,
                    duration_secs = excluded.duration_secs
                """,
                arguments: [
                    episode.id, episode.feedGuid, episode.subscriptionId,
                    episode.title, episode.description,
                    episode.pubDate, episode.durationSecs,
                    episode.enclosureUrl, episode.artworkUrl,
                    playbackStr, episode.playbackPositionSecs,
                    downloadStr, downloadProgress,
                    episode.isFlagged, episode.fileSizeBytes.map { Int64($0) }, episode.localPath,
                ],
            )
        }
        return .success
    }

    private func getEpisode(id: String) throws -> StorageResult {
        let row = try db.read { db -> Row? in
            return try Row.fetchOne(db, sql: "SELECT * FROM episodes WHERE id = ?", arguments: [id])
        }
        guard let row else { return .notFound }
        return .episode(Self.episode(from: row))
    }

    private func listEpisodesBySubscription(subscriptionId: String) throws -> StorageResult {
        let rows = try db.read { db -> [Row] in
            return try Row.fetchAll(
                db,
                sql: "SELECT * FROM episodes WHERE subscription_id = ? ORDER BY pub_date DESC",
                arguments: [subscriptionId],
            )
        }
        return .episodes(rows.map(Self.episode(from:)))
    }

    private func getEpisodeByFeedGuid(subscriptionId: String, feedGuid: String) throws -> StorageResult {
        let row = try db.read { db -> Row? in
            return try Row.fetchOne(
                db,
                sql: "SELECT * FROM episodes WHERE subscription_id = ? AND feed_guid = ?",
                arguments: [subscriptionId, feedGuid],
            )
        }
        guard let row else { return .notFound }
        return .episode(Self.episode(from: row))
    }

    private func updatePlaybackStatus(
        episodeId: String, status: PlaybackStatus, positionSecs: UInt32?,
    ) async throws -> StorageResult {
        let statusStr = Self.playbackStatusString(status)
        let position = positionSecs.map { Int32($0) }
        try await db.write { db in
            try db.execute(
                sql: """
                UPDATE episodes
                SET playback_status = ?, playback_position_secs = ?
                WHERE id = ?
                """,
                arguments: [statusStr, position, episodeId],
            )
        }
        return .success
    }
}

// MARK: - Row mapping + status conversion

private extension DatabaseManager {
    static func subscription(from row: Row) -> Subscription {
        Subscription(
            id: row["id"],
            feedUrl: row["feed_url"],
            title: row["title"],
            artworkUrl: row["artwork_url"],
            description: row["description"],
            lastRefreshed: row["last_refreshed"],
            createdAt: row["created_at"],
        )
    }

    static func episode(from row: Row) -> Episode {
        Episode(
            id: row["id"],
            feedGuid: row["feed_guid"],
            subscriptionId: row["subscription_id"],
            title: row["title"],
            description: row["description"],
            pubDate: row["pub_date"],
            durationSecs: (row["duration_secs"] as Int32?).map { UInt32(bitPattern: $0) },
            enclosureUrl: row["enclosure_url"],
            artworkUrl: row["artwork_url"],
            playbackStatus: playbackStatus(from: row["playback_status"]),
            playbackPositionSecs: (row["playback_position_secs"] as Int32?).map {
                UInt32(bitPattern: $0)
            },
            downloadStatus: downloadStatus(from: row["download_status"]),
            downloadProgress: (row["download_progress"] as Int32?).map { UInt8(truncatingIfNeeded: $0) },
            isFlagged: row["is_flagged"],
            fileSizeBytes: (row["file_size_bytes"] as Int64?).map { UInt64(bitPattern: $0) },
            localPath: row["local_path"],
        )
    }

    static func playbackStatusString(_ status: PlaybackStatus) -> String {
        switch status {
        case .unplayed: "Unplayed"
        case .inProgress: "InProgress"
        case .played: "Played"
        }
    }

    static func playbackStatus(from string: String) -> PlaybackStatus {
        switch string {
        case "Unplayed": .unplayed
        case "InProgress": .inProgress
        case "Played": .played
        default: fatalError(
                "Unknown PlaybackStatus in DB: '\(string)' — add a case to "
                    + "playbackStatus(from:) and playbackStatusString(_:)",
            )
        }
    }

    static func downloadStatusString(_ status: DownloadStatus) -> String {
        switch status {
        case .notDownloaded: "NotDownloaded"
        case .queued: "Queued"
        case .downloading: "Downloading"
        case .downloaded: "Downloaded"
        case .failed: "Failed"
        case .removedFromFeed: "RemovedFromFeed"
        }
    }

    static func downloadStatus(from string: String) -> DownloadStatus {
        switch string {
        case "NotDownloaded": .notDownloaded
        case "Queued": .queued
        case "Downloading": .downloading
        case "Downloaded": .downloaded
        case "Failed": .failed
        case "RemovedFromFeed": .removedFromFeed
        default: fatalError(
                "Unknown DownloadStatus in DB: '\(string)' — add a case to "
                    + "downloadStatus(from:) and downloadStatusString(_:)",
            )
        }
    }
}
