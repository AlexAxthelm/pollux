import SwiftUI

@main
struct PolluxApp: App {
    @StateObject private var core = Core()

    var body: some Scene {
        WindowGroup {
            ContentView(core: core)
        }
    }
}
