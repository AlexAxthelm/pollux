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
                (id, feed_url, title, artwork_url, description, last_refreshed, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(feed_url) DO UPDATE SET
                title = excluded.title,
                artwork_url = excluded.artwork_url,
                description = excluded.description,
                last_refreshed = excluded.last_refreshed
            """,
            arguments: [
                subscription.id, subscription.feedUrl, subscription.title,
                subscription.artworkUrl, subscription.description,
                subscription.lastRefreshed, subscription.createdAt,
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
                -- note(refresh): download_status/local_path are preserved on
                -- conflict, but file_size_bytes is overwritten with the feed's
                -- advertised <enclosure length>. For a downloaded episode this clobbers
                -- the real on-disk size that updateDownloadState recorded. There's no
                -- refresh feature yet, so it can't fire today; when feed refresh lands,
                -- stop overwriting file_size_bytes for rows whose download_status is
                -- Downloaded (or drop it from this SET and let the download path own it).
                file_size_bytes = excluded.file_size_bytes
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
