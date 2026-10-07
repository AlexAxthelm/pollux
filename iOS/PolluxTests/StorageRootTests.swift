import Foundation
import Testing

@testable import Pollux

@Suite struct StorageRootTests {
    @Test func isTheApplicationSupportDirectory() throws {
        let root = try #require(StorageRoot.applicationSupport())
        let expected = try #require(
            FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first,
        )
        #expect(root == expected)
    }

    @Test func downloadedFilesResolveUnderIt() throws {
        // Playback opens what downloads stored, so a stored (relative) path must resolve to the
        // same place whichever of them does the resolving.
        let root = try #require(StorageRoot.applicationSupport())
        let url = DownloadManager.absoluteURL(for: "Downloads/abc.mp3", root: root)
        #expect(url == root.appendingPathComponent("Downloads/abc.mp3"))
    }
}
