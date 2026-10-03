import App
import Foundation
import GRDB

/// Storage-layer errors that carry a specific message. Thrown (not returned)
/// from inside a write closure so GRDB rolls the transaction back.
enum DatabaseManagerError: Error, LocalizedError {
    case subscriptionMissingAfterUpsert(feedURL: String)

    var errorDescription: String? {
        switch self {
        case let .subscriptionMissingAfterUpsert(feedURL):
            "subscription disappeared after upsert for feed_url: \(feedURL)"
        }
    }
}

actor DatabaseManager {
    private let db: DatabasePool
    /// Wall clock, injectable so tests can move time (the removed-from-feed rule is
    /// time-based).
    private let now: @Sendable () -> Date

    init() throws {
        now = { Date() }
        guard let support = FileManager.default.urls(
            for: .applicationSupportDirectory, in: .userDomainMask,
        ).first else {
            fatalError("Application Support directory unavailable")
        }
        try FileManager.default.createDirectory(at: support, withIntermediateDirectories: true)
        db = try DatabasePool(path: support.appendingPathComponent("pollux.sqlite").path)
        try Self.runMigrations(db)
    }

    /// Accepts an explicit path — use this in tests to point at a temp file.
    init(path: String, now: @escaping @Sendable () -> Date = { Date() }) throws {
        self.now = now
        db = try DatabasePool(path: path)
        try Self.runMigrations(db)
    }

    // MARK: - Execute

    @discardableResult
    func execute(_ operation: StorageOperation) async throws -> StorageResult {
        // Split by domain so neither dispatcher trips the cyclomatic-complexity
        // limit as operations are added. Subscription/feed ops here; episode ops in
        // `executeEpisode`.
        switch operation {
        case .listSubscriptions:
            try listSubscriptions()
        case let .getSubscription(id):
            try getSubscription(id: id)
        case let .upsertSubscription(sub):
            try await upsertSubscription(sub)
        case let .deleteSubscription(id):
            try await deleteSubscription(id: id)
        case let .upsertFeedWithEpisodes(subscription, episodes):
            try await upsertFeedWithEpisodes(subscription: subscription, episodes: episodes)
        case let .updateRefreshState(subscriptionId, lastRefreshed, lastRefreshError, retryAfterUntil):
            try await updateRefreshState(
                subscriptionId: subscriptionId, lastRefreshed: lastRefreshed,
                lastRefreshError: lastRefreshError, retryAfterUntil: retryAfterUntil,
            )
        default:
            try await executeEpisode(operation)
        }
    }

    /// Episode-scoped storage operations. `execute` routes everything it does not
    /// handle here; the `default` is unreachable (every remaining case is covered).
    private func executeEpisode(_ operation: StorageOperation) async throws -> StorageResult {
        switch operation {
        case let .upsertEpisode(episode):
            try await upsertEpisode(episode)
        case let .getEpisode(id):
            try getEpisode(id: id)
        case let .listEpisodesBySubscription(subscriptionId):
            try listEpisodesBySubscription(subscriptionId: subscriptionId)
        case .listPendingDownloads:
            try listPendingDownloads()
        case let .getEpisodeByFeedGuid(subscriptionId, feedGuid):
            try getEpisodeByFeedGuid(subscriptionId: subscriptionId, feedGuid: feedGuid)
        case let .updatePlaybackStatus(episodeId, status, positionSecs):
            try await updatePlaybackStatus(episodeId: episodeId, status: status, positionSecs: positionSecs)
        case let .updateDownloadState(episodeId, status, localPath, sizeBytes):
            try await updateDownloadState(
                episodeId: episodeId, status: status, localPath: localPath, sizeBytes: sizeBytes,
            )
        default:
            fatalError("executeEpisode received a non-episode operation: \(operation)")
        }
    }

    // MARK: - Subscription operations

    private func listSubscriptions() throws -> StorageResult {
        let rows = try db.read { db -> [Row] in
            try Row.fetchAll(db, sql: "SELECT * FROM subscriptions ORDER BY title COLLATE NOCASE")
        }
        return .subscriptions(rows.map(Self.subscription(from:)))
    }

    private func getSubscription(id: String) throws -> StorageResult {
        let row = try db.read { db -> Row? in
            try Row.fetchOne(
                db, sql: "SELECT * FROM subscriptions WHERE id = ?", arguments: [id],
            )
        }
        guard let row else { return .notFound }
        return .subscription(Self.subscription(from: row))
    }

    private func upsertSubscription(_ sub: Subscription) async throws -> StorageResult {
        try await db.write { db in
            try Self.upsertSubscriptionRow(sub, db: db)
        }
        return .success
    }

    private func upsertFeedWithEpisodes(subscription: Subscription, episodes: [Episode]) async throws -> StorageResult {
        let nowSecs = Int64(now().timeIntervalSince1970)
        return try await db.write { db -> StorageResult in
            try Self.upsertSubscriptionRow(subscription, db: db)
            guard let subRow = try Row.fetchOne(
                db,
                sql: "SELECT * FROM subscriptions WHERE feed_url = ?",
                arguments: [subscription.feedUrl],
            ) else {
                // Throw rather than return: GRDB rolls back only on a thrown
                // error, so this undoes the subscription INSERT above instead of
                // committing it behind a reported failure. execute()'s caller
                // maps the thrown error to StorageResult.error.
                throw DatabaseManagerError.subscriptionMissingAfterUpsert(
                    feedURL: subscription.feedUrl,
                )
            }
            let canonical = Self.subscription(from: subRow)
            try Self.upsertFeedEpisodes(episodes, subscriptionId: canonical.id, now: nowSecs, db: db)
            return .subscription(canonical)
        }
    }

    /// Persists the outcome of a refresh that produced no new body (304, rate limit,
    /// or failure). `last_refreshed` is only moved when provided; the error and
    /// retry-after columns are written verbatim so a success clears them.
    private func updateRefreshState(
        subscriptionId: String, lastRefreshed: Int64?, lastRefreshError: String?,
        retryAfterUntil: Int64?,
    ) async throws -> StorageResult {
        try await db.write { db in
            try Self.updateRefreshStateRow(
                subscriptionId: subscriptionId, lastRefreshed: lastRefreshed,
                lastRefreshError: lastRefreshError, retryAfterUntil: retryAfterUntil, db: db,
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
        try await db.write { db in
            try Self.upsertEpisodeRow(episode, subscriptionId: episode.subscriptionId, db: db)
        }
        return .success
    }

    private func getEpisode(id: String) throws -> StorageResult {
        let row = try db.read { db -> Row? in
            try Row.fetchOne(db, sql: "SELECT * FROM episodes WHERE id = ?", arguments: [id])
        }
        guard let row else { return .notFound }
        return .episode(Self.episode(from: row))
    }

    private func listEpisodesBySubscription(subscriptionId: String) throws -> StorageResult {
        let rows = try db.read { db -> [Row] in
            try Row.fetchAll(
                db,
                sql: "SELECT * FROM episodes WHERE subscription_id = ? ORDER BY pub_date DESC",
                arguments: [subscriptionId],
            )
        }
        return .episodes(rows.map(Self.episode(from:)))
    }

    private func listPendingDownloads() throws -> StorageResult {
        // Known limitation: enqueue order isn't persisted, so a relaunch rebuilds the
        // queue by pub_date rather than the order the user queued them (and may start a
        // previously-Queued item before the one that was actively Downloading). Fine for
        // the serial MVP; revisit when the download queue gains explicit ordering — a
        // persisted sequence column, restoring the active item first — alongside the
        // downloads page / re-ordering / parallel-downloads work (see ROADMAP Phase 5).
        let rows = try db.read { db -> [Row] in
            try Row.fetchAll(
                db,
                sql: """
                SELECT * FROM episodes
                WHERE download_status IN ('Downloading', 'Queued')
                ORDER BY pub_date
                """,
            )
        }
        return .episodes(rows.map(Self.episode(from:)))
    }

    private func getEpisodeByFeedGuid(subscriptionId: String, feedGuid: String) throws -> StorageResult {
        let row = try db.read { db -> Row? in
            try Row.fetchOne(
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
        let position = positionSecs.map { Int64($0) }
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

    /// Persists a download-state transition. `localPath`/`sizeBytes` are written
    /// verbatim — passing nil clears the column to NULL, matching the core, which
    /// sends nil to wipe stale file metadata on delete/failure. Size overflow stores
    /// NULL (as in `upsertEpisodeRow`) rather than trapping.
    private func updateDownloadState(
        episodeId: String, status: DownloadStatus, localPath: String?, sizeBytes: UInt64?,
    ) async throws -> StorageResult {
        let statusStr = Self.downloadStatusString(status)
        let size = sizeBytes.flatMap { Int64(exactly: $0) }
        try await db.write { db in
            try db.execute(
                sql: """
                UPDATE episodes
                SET download_status = ?, local_path = ?, file_size_bytes = ?
                WHERE id = ?
                """,
                arguments: [statusStr, localPath, size, episodeId],
            )
        }
        return .success
    }
}
