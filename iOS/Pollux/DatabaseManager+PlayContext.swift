import App
import GRDB

/// The saved "active episode" the player restores at launch. Split from
/// `DatabaseManager` to keep that type under the body-length limit.
extension DatabaseManager {
    func savePlayContext(episodeId: String, source: EpisodeSource) async throws -> StorageResult {
        let columns = Self.episodeSourceColumns(source)
        try await db.write { db in
            try db.execute(
                sql: """
                INSERT OR REPLACE INTO play_context (id, episode_id, source_kind, source_id)
                VALUES (1, ?, ?, ?)
                """,
                arguments: [episodeId, columns.kind, columns.id],
            )
        }
        return .success
    }

    /// The saved active episode and where it was started from, or `.notFound` when none
    /// is saved.
    func loadPlayContext() throws -> StorageResult {
        let row = try db.read { db -> Row? in
            try Row.fetchOne(
                db,
                sql: """
                SELECT episodes.*, play_context.source_kind, play_context.source_id
                FROM play_context
                JOIN episodes ON episodes.id = play_context.episode_id
                WHERE play_context.id = 1
                """,
            )
        }
        guard let row else { return .notFound }
        let source = Self.episodeSource(
            kind: row["source_kind"], id: row["source_id"],
        )
        return .playContext(episode: Self.episode(from: row), source: source)
    }

    func clearPlayContext() async throws -> StorageResult {
        try await db.write { db in
            try db.execute(sql: "DELETE FROM play_context")
        }
        return .success
    }
}
