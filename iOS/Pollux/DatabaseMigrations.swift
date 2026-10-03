import Foundation
import GRDB

/// The schema history. Split out of `DatabaseManager` to keep the actor under the
/// type-length limit; migrations are append-only, so a database created at any earlier
/// version upgrades step by step. A migration that has shipped (or merely run on someone's
/// device) is never edited, only followed by a new one.
extension DatabaseManager {
    static func runMigrations(_ db: DatabasePool) throws {
        var migrator = DatabaseMigrator()
        registerInitialSchema(&migrator)
        registerRefreshSchema(&migrator)
        try migrator.migrate(db)
    }

    private static func registerInitialSchema(_ migrator: inout DatabaseMigrator) {
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
    }

    /// Everything feed refresh needed: validators and failure state on subscriptions, and
    /// the removed-from-feed bookkeeping on episodes.
    private static func registerRefreshSchema(_ migrator: inout DatabaseMigrator) {
        // Refresh bookkeeping: conditional-GET validators, the last refresh failure, and
        // the time before which a rate-limited host asked not to be hit again.
        migrator.registerMigration("v2_refresh") { db in
            try db.alter(table: "subscriptions") { t in
                t.add(column: "etag", .text)
                t.add(column: "last_modified", .text)
                t.add(column: "last_refresh_error", .text)
                t.add(column: "retry_after_until", .integer)
            }
        }
        // A counter of consecutive refreshes an episode was missing from its feed. Superseded
        // by v4, but stays registered because databases may already have run it.
        migrator.registerMigration("v3_missing_refreshes") { db in
            try db.alter(table: "episodes") { t in
                t.add(column: "missing_refreshes", .integer).notNull().defaults(to: 0)
            }
        }
        // Replaces the v3 counter with the time an episode was first missed, so the
        // removed-from-feed rule can be about elapsed wall time rather than a number of
        // refreshes (several quick refreshes during one stale-cache incident must not be
        // enough). Any in-progress counts are simply restarted. Storage-only: the core
        // never sees it.
        migrator.registerMigration("v4_missing_since") { db in
            try db.alter(table: "episodes") { t in
                t.add(column: "missing_since", .integer)
            }
            try db.alter(table: "episodes") { t in
                t.drop(column: "missing_refreshes")
            }
        }
    }
}
