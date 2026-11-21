import App
import GRDB

/// The user's saved settings. The JSON is the core's: this only stores it, one row per
/// level, and hands it back exactly as saved. Split from `DatabaseManager` to keep that
/// type under the body-length limit.
extension DatabaseManager {
    func saveSettings(scope: SettingsScope, json: String) async throws -> StorageResult {
        let columns = Self.settingsScopeColumns(scope)
        try await db.write { db in
            try db.execute(
                sql: """
                INSERT OR REPLACE INTO settings (scope_kind, scope_id, json)
                VALUES (?, ?, ?)
                """,
                arguments: [columns.kind, columns.id, json],
            )
        }
        return .success
    }

    /// Every saved settings document. A row whose level this build doesn't know (written by a
    /// newer one) is left in place and not returned, so it isn't lost.
    func loadAllSettings() throws -> StorageResult {
        let rows = try db.read { db in
            try Row.fetchAll(db, sql: "SELECT scope_kind, scope_id, json FROM settings")
        }
        let saved = rows.compactMap { row -> SavedSettings? in
            let kind: String = row["scope_kind"]
            let id: String = row["scope_id"]
            guard let scope = Self.settingsScope(kind: kind, id: id) else { return nil }
            return SavedSettings(scope: scope, json: row["json"])
        }
        return .allSettings(saved)
    }

    /// A `SettingsScope` as the `(kind, id)` pair stored in `settings`.
    static func settingsScopeColumns(_ scope: SettingsScope) -> (kind: String, id: String) {
        switch scope {
        case .global: ("global", "")
        case let .subscription(id): ("subscription", id)
        }
    }

    /// The inverse of `settingsScopeColumns`, or `nil` for a level this build doesn't know.
    static func settingsScope(kind: String, id: String) -> SettingsScope? {
        switch kind {
        case "global": .global
        case "subscription": .subscription(id)
        default: nil
        }
    }
}
