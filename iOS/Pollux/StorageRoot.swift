import Foundation

/// Where the app keeps its data: Application Support. The database, the downloaded audio
/// and the player (which opens the downloaded files) must all agree on this, so it is
/// looked up in one place.
enum StorageRoot {
    /// The Application Support directory, or nil if the system can't provide one.
    static func applicationSupport() -> URL? {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
    }
}
