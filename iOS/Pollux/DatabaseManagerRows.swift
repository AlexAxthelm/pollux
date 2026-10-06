import App
import Foundation
import GRDB

/// Row mapping + status conversion for `DatabaseManager`. Split out of the main file
/// to keep each under the file-length limit; these are module-internal helpers, not
/// part of any public surface.
extension DatabaseManager {
    static func upsertSubscriptionRow(_ subscription: Subscription, db: Database) throws {
        try db.execute(
            sql: """
            INSERT INTO subscriptions
                (id, feed_url, title, artwork_url, description, last_refreshed, created_at,
                 etag, last_modified, last_refresh_error, retry_after_until)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(feed_url) DO UPDATE SET
                title = excluded.title,
                artwork_url = excluded.artwork_url,
                description = excluded.description,
                last_refreshed = excluded.last_refreshed,
                etag = excluded.etag,
                last_modified = excluded.last_modified,
                last_refresh_error = excluded.last_refresh_error,
                retry_after_until = excluded.retry_after_until
            """,
            arguments: [
                subscription.id, subscription.feedUrl, subscription.title,
                subscription.artworkUrl, subscription.description,
                subscription.lastRefreshed, subscription.createdAt,
                subscription.etag, subscription.lastModified,
                subscription.lastRefreshError, subscription.retryAfterUntil,
            ],
        )
    }

    static func upsertEpisodeRow(_ episode: Episode, subscriptionId: String, db: Database) throws {
        let playbackStr = playbackStatusString(episode.playbackStatus)
        let downloadStr = downloadStatusString(episode.downloadStatus)
        try db.execute(
            sql: """
            INSERT INTO episodes
                (id, feed_guid, subscription_id, title, description, pub_date,
                 duration_secs, enclosure_url, artwork_url, playback_status,
                 playback_position_secs, download_status,
                 is_flagged, file_size_bytes, local_path)
            VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
            ON CONFLICT(subscription_id, feed_guid) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                enclosure_url = excluded.enclosure_url,
                artwork_url = excluded.artwork_url,
                pub_date = excluded.pub_date,
                duration_secs = excluded.duration_secs,
                -- A downloaded episode's size is the real on-disk size recorded by
                -- updateDownloadState; the feed's advertised <enclosure length> must not
                -- clobber it. Otherwise the feed's value is the freshest we have.
                file_size_bytes = CASE
                    WHEN episodes.download_status = 'Downloaded' THEN episodes.file_size_bytes
                    ELSE excluded.file_size_bytes
                END,
                -- An episode that dropped out of the feed and came back is available again.
                download_status = CASE
                    WHEN episodes.download_status = 'RemovedFromFeed' THEN 'NotDownloaded'
                    ELSE episodes.download_status
                END,
                -- Present in this feed, so it is no longer missing (see startMissClock).
                missing_since = NULL
            """,
            arguments: [
                episode.id, episode.feedGuid, subscriptionId,
                episode.title, episode.description,
                episode.pubDate, episode.durationSecs,
                episode.enclosureUrl, episode.artworkUrl,
                playbackStr, episode.playbackPositionSecs,
                downloadStr,
                episode.isFlagged,
                episode.fileSizeBytes.flatMap { Int64(exactly: $0) },
                episode.localPath,
            ],
        )
    }

    static func updateRefreshStateRow(
        subscriptionId: String, lastRefreshed: Int64?, lastRefreshError: String?,
        retryAfterUntil: Int64?, db: Database,
    ) throws {
        try db.execute(
            sql: """
            UPDATE subscriptions
            SET last_refreshed = COALESCE(?, last_refreshed),
                last_refresh_error = ?,
                retry_after_until = ?
            WHERE id = ?
            """,
            arguments: [lastRefreshed, lastRefreshError, retryAfterUntil, subscriptionId],
        )
    }

    /// How long an episode must have been continuously absent from its feed, in wall time,
    /// before it is flagged `RemovedFromFeed`. Flagging on a single absence would let one
    /// truncated or stale response (a CDN glitch, a feed briefly serving only its newest
    /// items) mark most of a feed removed, leaving those episodes un-downloadable until
    /// they reappear. Counting refreshes isn't enough either: a few quick manual refreshes
    /// during the same incident would all see the same bad response. Elapsed time is:
    /// an episode is only flagged if a refresh at least this long after it was first
    /// missed still doesn't see it. A feed that really dropped an episode is flagged by
    /// the first refresh after the window (a latest-only feed included).
    static let removalGraceSeconds: Int64 = 48 * 3600

    /// Writes a feed's episodes and reconciles the ones it no longer lists: start a miss
    /// clock for every stored episode that doesn't have one, upsert (which clears the clock
    /// of each episode still in the feed, leaving only the absent ones running), then flag
    /// those whose clock has run for the grace period. An empty list is far more likely a
    /// broken response than every episode being deleted, so it neither starts a clock nor
    /// flags anything.
    static func upsertFeedEpisodes(
        _ episodes: [Episode], subscriptionId: String, now: Int64, db: Database,
    ) throws {
        let reconcile = !episodes.isEmpty
        if reconcile {
            try startMissClock(subscriptionId: subscriptionId, now: now, db: db)
        }
        for episode in episodes {
            try upsertEpisodeRow(episode, subscriptionId: subscriptionId, db: db)
        }
        if reconcile {
            try flagEpisodesRemovedFromFeed(subscriptionId: subscriptionId, now: now, db: db)
        }
    }

    /// Step one of reconciling a refresh: record `now` as the time each of the feed's
    /// episodes was first missed, unless a clock is already running. The episode upsert
    /// that follows clears it for each one still in the feed, so only the absent episodes
    /// keep it. A clock in the future (the wall clock was ahead and has since been
    /// corrected) is restarted at `now`; left alone it could hold an episode back, or
    /// release it early, by as much as the clock error.
    static func startMissClock(subscriptionId: String, now: Int64, db: Database) throws {
        try db.execute(
            sql: """
            UPDATE episodes
            SET missing_since = CASE
                WHEN missing_since IS NULL OR missing_since > ? THEN ?
                ELSE missing_since
            END
            WHERE subscription_id = ?
            """,
            arguments: [now, now, subscriptionId],
        )
    }

    /// Step two: flag episodes that have been missing for at least the grace period. Only
    /// rows with nothing to lose are flagged (`NotDownloaded`/`Failed`): a downloaded,
    /// queued, or in-flight episode keeps its state so its file stays playable and its
    /// download isn't orphaned. Its clock keeps running, so if the file is deleted later
    /// the next refresh flags it.
    static func flagEpisodesRemovedFromFeed(subscriptionId: String, now: Int64, db: Database) throws {
        try db.execute(
            sql: """
            UPDATE episodes
            SET download_status = 'RemovedFromFeed'
            WHERE subscription_id = ?
              AND missing_since IS NOT NULL
              AND missing_since <= ?
              AND download_status IN ('NotDownloaded', 'Failed')
            """,
            arguments: [subscriptionId, now - removalGraceSeconds],
        )
    }

    static func subscription(from row: Row) -> Subscription {
        Subscription(
            id: row["id"],
            feedUrl: row["feed_url"],
            title: row["title"],
            artworkUrl: row["artwork_url"],
            description: row["description"],
            lastRefreshed: row["last_refreshed"],
            createdAt: row["created_at"],
            etag: row["etag"],
            lastModified: row["last_modified"],
            lastRefreshError: row["last_refresh_error"],
            retryAfterUntil: row["retry_after_until"],
        )
    }

    static func episode(from row: Row) -> Episode {
        // Checked conversions: a stored value outside the target type's range
        // (written by a later schema or external tooling) reads back as nil
        // rather than trapping or silently wrapping — matching the write path,
        // which stores NULL on overflow (see fileSizeBytes in upsertEpisodeRow).
        Episode(
            id: row["id"],
            feedGuid: row["feed_guid"],
            subscriptionId: row["subscription_id"],
            title: row["title"],
            description: row["description"],
            pubDate: row["pub_date"],
            durationSecs: (row["duration_secs"] as Int64?).flatMap { UInt32(exactly: $0) },
            enclosureUrl: row["enclosure_url"],
            artworkUrl: row["artwork_url"],
            playbackStatus: playbackStatus(from: row["playback_status"]),
            playbackPositionSecs: (row["playback_position_secs"] as Int64?).flatMap { UInt32(exactly: $0) },
            downloadStatus: downloadStatus(from: row["download_status"]),
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
