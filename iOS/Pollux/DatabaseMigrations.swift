import Foundation
import GRDB

/// The schema. Split out of `DatabaseManager` to keep the actor under the type-length limit.
///
/// **Until the first release the database is disposable, so the schema is a single baseline
/// that is edited in place** rather than grown by appending migrations. In debug builds the
/// migrator erases and recreates the database whenever that definition changes, so an
/// existing simulator database never needs wiping by hand.
///
/// Before the first release, remove the erase-on-change below and freeze `v1_initial`. From
/// then on a schema change is a *new* append-only migration, and each one that alters a
/// table that can already hold rows should be tested against a database that has some (the
/// migrator can stop at an earlier version with `migrate(_:upTo:)`).
extension DatabaseManager {
    static func runMigrations(_ db: DatabasePool) throws {
        var migrator = DatabaseMigrator()
        #if DEBUG
            migrator.eraseDatabaseOnSchemaChange = true
        #endif
        registerBaselineSchema(&migrator)
        try migrator.migrate(db)
    }

    private static func registerBaselineSchema(_ migrator: inout DatabaseMigrator) {
        migrator.registerMigration("v1_initial") { db in
            try db.create(table: "subscriptions") { t in
                t.column("id", .text).primaryKey()
                t.column("feed_url", .text).notNull().unique()
                t.column("title", .text).notNull()
                t.column("artwork_url", .text)
                t.column("description", .text)
                t.column("last_refreshed", .integer)
                t.column("created_at", .integer).notNull()
                // Feed refresh: the conditional-GET validators from the last successful
                // fetch, why the last refresh failed, and the time before which automatic
                // refresh leaves the feed alone (a rate-limited host, or a failure backoff).
                t.column("etag", .text)
                t.column("last_modified", .text)
                t.column("last_refresh_error", .text)
                t.column("retry_after_until", .integer)
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
                // Unix time a refresh first failed to find this episode in its feed, or NULL
                // while it is present. Storage-only (the core never sees it): the
                // removed-from-feed rule flags an episode once this is old enough.
                t.column("missing_since", .integer)
                t.uniqueKey(["subscription_id", "feed_guid"])
            }
            try db.create(
                index: "episodes_subscription_id",
                on: "episodes",
                columns: ["subscription_id"],
            )
            // The episode the player was on, so the mini-player can be restored at
            // launch. Single row (id pinned to 1); position lives on the episode itself.
            // Deleting the episode (via its subscription) drops the row with it.
            try db.create(table: "play_context") { t in
                t.column("id", .integer).primaryKey().check(sql: "id = 1")
                t.column("episode_id", .text).notNull()
                    .references("episodes", column: "id", onDelete: .cascade)
            }
        }
    }
}
