import App
import Foundation
import GRDB
@testable import Pollux
import Testing

private func makeManager(path: String? = nil) throws -> DatabaseManager {
    let path = path ?? FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString + ".sqlite").path
    return try DatabaseManager(path: path)
}

private func loadedAll(_ db: DatabaseManager) async throws -> [SavedSettings] {
    guard case let .allSettings(saved) = try await db.execute(.loadAllSettings) else {
        Issue.record("Expected .allSettings")
        return []
    }
    return saved
}

private func loadedJSON(_ db: DatabaseManager, _ scope: SettingsScope) async throws -> String? {
    try await loadedAll(db).first { $0.scope == scope }?.json
}

struct SettingsStorageTests {
    @Test func `nothing saved loads as an empty list`() async throws {
        let db = try makeManager()
        #expect(try await loadedAll(db).isEmpty)
    }

    @Test func `saved settings come back exactly as saved`() async throws {
        let db = try makeManager()
        let json = #"{"skip_forward_secs":60}"#
        let result = try await db.execute(.saveSettings(scope: .global, json: json))
        guard case .success = result else {
            Issue.record("Expected .success, got \(result)")
            return
        }
        #expect(try await loadedJSON(db, .global) == json)
    }

    @Test func `saving replaces the previous settings`() async throws {
        let db = try makeManager()
        try await db.execute(.saveSettings(scope: .global, json: #"{"skip_back_secs":10}"#))
        try await db.execute(.saveSettings(scope: .global, json: "{}"))
        #expect(try await loadedJSON(db, .global) == "{}")
        #expect(try await loadedAll(db).count == 1)
    }

    @Test func `each level is kept separately`() async throws {
        let db = try makeManager()
        try await db.execute(.saveSettings(scope: .global, json: #"{"skip_forward_secs":45}"#))
        try await db.execute(.saveSettings(scope: .subscription("a"), json: #"{"skip_back_secs":10}"#))
        try await db.execute(.saveSettings(scope: .subscription("b"), json: #"{"skip_back_secs":5}"#))
        #expect(try await loadedAll(db).count == 3)
        #expect(try await loadedJSON(db, .subscription("a")) == #"{"skip_back_secs":10}"#)
        #expect(try await loadedJSON(db, .subscription("b")) == #"{"skip_back_secs":5}"#)
        #expect(try await loadedJSON(db, .global) == #"{"skip_forward_secs":45}"#)
    }

    @Test func `settings survive reopening the database`() async throws {
        let path = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString + ".sqlite").path
        let json = #"{"resume_rewind_secs":0}"#
        try await makeManager(path: path).execute(.saveSettings(scope: .subscription("a"), json: json))
        #expect(try await loadedJSON(makeManager(path: path), .subscription("a")) == json)
    }

    @Test func `deleting A subscription removes its settings only`() async throws {
        let db = try makeManager()
        try await db.execute(.saveSettings(scope: .global, json: "{}"))
        try await db.execute(.saveSettings(scope: .subscription("a"), json: "{}"))
        try await db.execute(.saveSettings(scope: .subscription("b"), json: "{}"))
        try await db.execute(.deleteSubscription(id: "a"))
        let scopes = try await loadedAll(db).map(\.scope)
        #expect(scopes.count == 2)
        #expect(scopes.contains(.global))
        #expect(scopes.contains(.subscription("b")))
        #expect(!scopes.contains(.subscription("a")))
    }

    @Test func `a level this build does not know is left alone and not returned`() async throws {
        let db = try makeManager()
        try await db.execute(.saveSettings(scope: .global, json: "{}"))
        try await db.db.write { db in
            try db.execute(
                sql: "INSERT INTO settings (scope_kind, scope_id, json) VALUES ('playlist', 'p', '{}')",
            )
        }
        #expect(try await loadedAll(db).map(\.scope) == [.global])
        let remaining = try await db.db.read { db in
            try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM settings WHERE scope_kind = 'playlist'")
        }
        #expect(remaining == 1)
    }
}
