import App
import Foundation
import GRDB
import Testing

@testable import Pollux

private func freshPath() -> String {
    FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString + ".sqlite").path
}

private func columns(of table: String, at path: String) throws -> Set<String> {
    let pool = try DatabasePool(path: path)
    let rows = try pool.read { db in
        try Row.fetchAll(db, sql: "PRAGMA table_info(\(table))")
    }
    return Set(rows.map { $0["name"] as String })
}

@Suite("Database baseline")
struct DatabaseBaselineTests {
    @Test func theBaselineHasTheRefreshColumns() throws {
        let path = freshPath()
        _ = try DatabaseManager(path: path)

        let subscriptions = try columns(of: "subscriptions", at: path)
        #expect(subscriptions.isSuperset(of: ["etag", "last_modified", "last_refresh_error", "retry_after_until"]))
        #expect(try columns(of: "episodes", at: path).contains("missing_since"))
    }

    #if DEBUG
        @Test func aDatabaseFromAnOlderBaselineIsRecreatedNotLeftBroken() async throws {
            // Until the first release the schema is edited in place under the same
            // identifier. A simulator database created by an earlier baseline must come up
            // on the current one in debug builds instead of failing on a missing column
            // (which would crash the app at launch).
            let path = freshPath()
            let old = try DatabasePool(path: path)
            var oldMigrator = DatabaseMigrator()
            oldMigrator.registerMigration("v1_initial") { db in
                // The schema before feed refresh existed.
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
                    t.column("enclosure_url", .text).notNull()
                    t.uniqueKey(["subscription_id", "feed_guid"])
                }
            }
            try oldMigrator.migrate(old)
            try await old.write { db in
                try db.execute(
                    sql: "INSERT INTO subscriptions (id, feed_url, title, created_at) VALUES ('s', 'u', 't', 1)",
                )
            }
            try old.close()

            let manager = try DatabaseManager(path: path) // must not throw

            #expect(try columns(of: "subscriptions", at: path).contains("etag"))
            #expect(try columns(of: "episodes", at: path).contains("missing_since"))
            // And it is usable: the old rows were discarded with the old schema.
            let listed = try await manager.execute(.listSubscriptions)
            guard case let .subscriptions(rows) = listed else {
                Issue.record("Expected .subscriptions, got \(listed)")
                return
            }
            #expect(rows.isEmpty, "the disposable database was recreated")
        }
    #endif
}
