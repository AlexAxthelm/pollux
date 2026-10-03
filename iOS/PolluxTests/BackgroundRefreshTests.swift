import Foundation
import Testing

@testable import Pollux

@Suite("BackgroundRefresh")
struct BackgroundRefreshTests {
    @Test func earliestBeginIsThreeHoursOut() {
        let now = Date(timeIntervalSince1970: 1_000_000)
        #expect(BackgroundRefresh.earliestBeginDate(from: now) == now.addingTimeInterval(3 * 3600))
    }

    @Test func identifierIsDeclaredInTheAppBundle() throws {
        // The system silently refuses to schedule an identifier that isn't declared in
        // Info.plist, so drift between the two would quietly disable background refresh.
        let app = try #require(Bundle(for: Core.self) as Bundle?)
        let declared = app.object(forInfoDictionaryKey: "BGTaskSchedulerPermittedIdentifiers") as? [String]
        #expect(declared?.contains(BackgroundRefresh.identifier) == true)
        let modes = app.object(forInfoDictionaryKey: "UIBackgroundModes") as? [String]
        #expect(modes?.contains("fetch") == true)
    }
}
