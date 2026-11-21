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

    /// The saved settings JSON for `scope`, or `.notFound` when none is saved.
    func loadSettings(scope: SettingsScope) throws -> StorageResult {
        let columns = Self.settingsScopeColumns(scope)
        let json = try db.read { db -> String? in
            try String.fetchOne(
                db,
                sql: "SELECT json FROM settings WHERE scope_kind = ? AND scope_id = ?",
                arguments: [columns.kind, columns.id],
            )
        }
        guard let json else { return .notFound }
        return .settings(json)
    }

    /// A `SettingsScope` as the `(kind, id)` pair stored in `settings`.
    static func settingsScopeColumns(_ scope: SettingsScope) -> (kind: String, id: String) {
        switch scope {
        case .global: ("global", "")
        }
    }
}
