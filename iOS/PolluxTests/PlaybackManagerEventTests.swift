import App
import AVFoundation
import Foundation
import Testing

@testable import Pollux

// What the engine tells the core while an item plays: position ticks, the end of the file,
// failures. Nothing is actually played: ticks are driven through `reportTick`, and the
// system's end and failure notifications are posted for the loaded item by hand.

extension EngineRig {
    /// A file under the storage root that is not audio, so the engine can't open it.
    func installGarbage(named name: String = "garbage.mp3") throws -> String {
        let directory = root.appendingPathComponent("Downloads", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try Data("this is not an audio file".utf8).write(to: directory.appendingPathComponent(name))
        return "Downloads/\(name)"
    }
}

@MainActor
private func load(_ rig: EngineRig, session: UInt32, path: String, start: UInt32 = 0) {
    _ = rig.manager.perform(
        .load(session: session, media: .local(localPath: path), startSecs: start, autoplay: false),
    )
}

/// The ticks the core was sent, as (session, position).
@MainActor
private func ticks(_ rig: EngineRig) -> [[UInt32]] {
    rig.log.events.compactMap {
        if case let .playerTick(session, position) = $0 { return [session, position] }
        return nil
    }
}

@MainActor
private func endNotification(for item: AVPlayerItem?) {
    NotificationCenter.default.post(name: AVPlayerItem.didPlayToEndTimeNotification, object: item)
}

@MainActor
private func failureNotification(for item: AVPlayerItem?, message: String?) {
    let userInfo = message.map {
        [AVPlayerItemFailedToPlayToEndTimeErrorKey: NSError(
            domain: "test", code: 1, userInfo: [NSLocalizedDescriptionKey: $0],
        )]
    }
    NotificationCenter.default.post(
        name: AVPlayerItem.failedToPlayToEndTimeNotification, object: item, userInfo: userInfo,
    )
}

@Suite @MainActor struct PlaybackManagerEventTests {
    // MARK: - Position ticks

    @Test func aTickIsReportedOnceTheLoadHasSettled() async throws {
        let rig = makeEngineRig()
        load(rig, session: 3, path: try rig.installAudio())
        try await waitUntil { rig.manager.pendingSeeks == 0 }

        rig.manager.reportTick(seconds: 12.9, isPlaying: true)

        #expect(ticks(rig) == [[3, 12]])
    }

    @Test func ticksAreHeldBackWhileASeekIsPending() async throws {
        let rig = makeEngineRig()
        load(rig, session: 3, path: try rig.installAudio())
        try await waitUntil { rig.manager.pendingSeeks == 0 }

        // A seek is landing: the engine still reports the old spot, which would snap the
        // scrubber back if it were passed on.
        rig.manager.pendingSeeks = 1
        rig.manager.reportTick(seconds: 55, isPlaying: true)
        #expect(ticks(rig).isEmpty)

        // Two overlapping seeks: still held until the last has landed.
        rig.manager.pendingSeeks = 2
        rig.manager.reportTick(seconds: 55, isPlaying: true)
        #expect(ticks(rig).isEmpty)

        rig.manager.pendingSeeks = 0
        rig.manager.reportTick(seconds: 1, isPlaying: true)
        #expect(ticks(rig) == [[3, 1]])
    }

    @Test func everySeekIsAccountedForOnceItLands() async throws {
        let rig = makeEngineRig()
        // A load seeks to where to start; so do explicit seeks, some of them overlapping.
        load(rig, session: 3, path: try rig.installAudio(), start: 1)
        _ = rig.manager.perform(.seek(secs: 1))
        _ = rig.manager.perform(.seek(secs: 0))
        _ = rig.manager.perform(.seek(secs: 1))

        try await waitUntil { rig.manager.pendingSeeks == 0 }
        // Settled at zero, not stuck above it (ticks would be held back for good) and not
        // below it (a later seek would then let stale ticks through).
        try await Task.sleep(for: .milliseconds(200))
        #expect(rig.manager.pendingSeeks == 0)
    }

    @Test func noTickIsReportedWhileNotPlaying() async throws {
        let rig = makeEngineRig()
        load(rig, session: 3, path: try rig.installAudio())
        try await waitUntil { rig.manager.pendingSeeks == 0 }

        rig.manager.reportTick(seconds: 5, isPlaying: false)

        #expect(ticks(rig).isEmpty)
    }

    @Test func noTickIsReportedWithNothingLoaded() {
        let rig = makeEngineRig()

        rig.manager.reportTick(seconds: 5, isPlaying: true)

        #expect(ticks(rig).isEmpty)
    }

    @Test func noTickIsReportedAfterTheItemIsUnloaded() async throws {
        let rig = makeEngineRig()
        load(rig, session: 3, path: try rig.installAudio())
        try await waitUntil { rig.manager.pendingSeeks == 0 }
        _ = rig.manager.perform(.stop)

        rig.manager.reportTick(seconds: 5, isPlaying: true)

        #expect(ticks(rig).isEmpty)
    }

    @Test func aPositionThatIsNotATimeIsNotReported() async throws {
        let rig = makeEngineRig()
        load(rig, session: 3, path: try rig.installAudio())
        try await waitUntil { rig.manager.pendingSeeks == 0 }

        for bad in [Double.nan, .infinity, -1, 1e300] {
            rig.manager.reportTick(seconds: bad, isPlaying: true)
        }

        #expect(ticks(rig).isEmpty)
    }

    // MARK: - End of the file and failures

    @Test func theEndOfTheFileIsReportedWithItsSession() throws {
        let rig = makeEngineRig()
        load(rig, session: 4, path: try rig.installAudio())

        endNotification(for: rig.manager.loadedItem)

        #expect(rig.log.events.contains(.playerEnded(session: 4)))
    }

    @Test func aFailureMidPlayIsReportedWithTheSystemsMessage() throws {
        let rig = makeEngineRig()
        load(rig, session: 4, path: try rig.installAudio())

        failureNotification(for: rig.manager.loadedItem, message: "network lost")

        #expect(rig.log.events.contains(.playerFailed(session: 4, message: "network lost")))
    }

    @Test func aFailureWithNoMessageStillSaysSomething() throws {
        let rig = makeEngineRig()
        load(rig, session: 4, path: try rig.installAudio())

        failureNotification(for: rig.manager.loadedItem, message: nil)

        #expect(rig.log.events.contains(
            .playerFailed(session: 4, message: "Playback stopped unexpectedly"),
        ))
    }

    @Test func aFileTheEngineCantOpenIsReportedUnusable() async throws {
        let rig = makeEngineRig()
        load(rig, session: 6, path: try rig.installGarbage())

        try await waitUntil {
            rig.log.events.contains {
                if case let .playerMediaUnusable(session, _) = $0 { return session == 6 }
                return false
            }
        }
    }

    // MARK: - Items that are no longer current

    @Test func theEndOfAReplacedItemIsIgnored() throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()
        load(rig, session: 1, path: path)
        let replaced = rig.manager.loadedItem
        load(rig, session: 2, path: path)

        endNotification(for: replaced)
        #expect(!rig.log.events.contains(.playerEnded(session: 1)))

        endNotification(for: rig.manager.loadedItem)
        #expect(rig.log.events.contains(.playerEnded(session: 2)))
    }

    @Test func theEndOfAnUnloadedItemIsIgnored() throws {
        let rig = makeEngineRig()
        load(rig, session: 5, path: try rig.installAudio())
        let item = rig.manager.loadedItem
        _ = rig.manager.perform(.stop)

        endNotification(for: item)
        failureNotification(for: item, message: "late")

        #expect(rig.log.events.isEmpty || !rig.log.events.contains(.playerEnded(session: 5)))
        #expect(!rig.log.events.contains(.playerFailed(session: 5, message: "late")))
    }

    @Test func theEndOfAnItemUnloadedByAFailedLoadIsIgnored() throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()
        load(rig, session: 7, path: path)
        let item = rig.manager.loadedItem
        // The next load fails up front, which unloads the engine.
        _ = rig.manager.perform(
            .load(session: 8, media: .local(localPath: "Downloads/missing.mp3"), startSecs: 0, autoplay: false),
        )

        endNotification(for: item)

        #expect(!rig.log.events.contains(.playerEnded(session: 7)))
    }
}
