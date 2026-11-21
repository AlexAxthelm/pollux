import App
import Foundation
import Testing

@testable import Pollux

private func makeManager(path: String? = nil) throws -> DatabaseManager {
    let path = path ?? FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString + ".sqlite").path
    return try DatabaseManager(path: path)
}

private func loadedJSON(_ db: DatabaseManager) async throws -> String? {
    guard case let .settings(json) = try await db.execute(.loadSettings(scope: .global)) else {
        return nil
    }
    return json
}

@Suite struct SettingsStorageTests {
    @Test func nothingSavedIsNotFound() async throws {
        let db = try makeManager()
        let result = try await db.execute(.loadSettings(scope: .global))
        guard case .notFound = result else {
            Issue.record("Expected .notFound, got \(result)")
            return
        }
    }

    @Test func savedSettingsComeBackExactlyAsSaved() async throws {
        let db = try makeManager()
        let json = #"{"skip_forward_secs":60}"#
        let result = try await db.execute(.saveSettings(scope: .global, json: json))
        guard case .success = result else {
            Issue.record("Expected .success, got \(result)")
            return
        }
        #expect(try await loadedJSON(db) == json)
    }

    @Test func savingReplacesThePreviousSettings() async throws {
        let db = try makeManager()
        try await db.execute(.saveSettings(scope: .global, json: #"{"skip_back_secs":10}"#))
        try await db.execute(.saveSettings(scope: .global, json: "{}"))
        #expect(try await loadedJSON(db) == "{}")
    }

    @Test func settingsSurviveReopeningTheDatabase() async throws {
        let path = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString + ".sqlite").path
        let json = #"{"resume_rewind_secs":0}"#
        try await makeManager(path: path).execute(.saveSettings(scope: .global, json: json))
        #expect(try await loadedJSON(makeManager(path: path)) == json)
    }
}
