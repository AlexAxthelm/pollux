import App
import AVFoundation
import Foundation
import Testing

@testable import Pollux

// These drive the real `PlaybackManager` (and so a real `AVPlayer`) against files in a
// temporary storage root. Nothing is played: loads are paused (`autoplay: false`), which is
// enough to see what the engine has loaded.

/// One second of silence as a minimal PCM WAV file, the smallest valid audio `AVPlayer`
/// will open.
func silentWAV() -> Data {
    let sampleRate: UInt32 = 8000
    let payload = Data(count: Int(sampleRate)) // 8-bit mono: one byte per sample
    var wav = Data()
    func append(_ value: UInt32) {
        withUnsafeBytes(of: value.littleEndian) { wav.append(contentsOf: $0) }
    }
    func append(_ value: UInt16) {
        withUnsafeBytes(of: value.littleEndian) { wav.append(contentsOf: $0) }
    }
    wav.append(Data("RIFF".utf8))
    append(UInt32(36 + payload.count))
    wav.append(Data("WAVEfmt ".utf8))
    append(UInt32(16)) // fmt chunk size
    append(UInt16(1)) // PCM
    append(UInt16(1)) // mono
    append(sampleRate)
    append(sampleRate) // byte rate
    append(UInt16(1)) // block align
    append(UInt16(8)) // bits per sample
    wav.append(Data("data".utf8))
    append(UInt32(payload.count))
    wav.append(payload)
    return wav
}

/// Stands in for claiming the audio session, so tests can see when it is claimed (which
/// takes audio focus from other apps) and make it fail.
final class AudioSessionProbe: @unchecked Sendable {
    struct Refused: Error, LocalizedError {
        var errorDescription: String? {
            "audio is busy"
        }
    }

    var claims = 0
    var refuse = false

    func claim() throws {
        claims += 1
        if refuse { throw Refused() }
    }
}

/// Collects what the engine reports to the core.
@MainActor
final class EventLog {
    private(set) var events: [Event] = []

    func record(_ event: Event) {
        events.append(event)
    }
}

@MainActor
struct EngineRig {
    let manager: PlaybackManager
    let root: URL
    let audio: AudioSessionProbe
    let log: EventLog

    /// A relative path (as the core stores them) to a real, playable file under the root.
    func installAudio(named name: String = "silence.wav") throws -> String {
        let directory = root.appendingPathComponent("Downloads", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try silentWAV().write(to: directory.appendingPathComponent(name))
        return "Downloads/\(name)"
    }
}

@MainActor
func makeEngineRig() -> EngineRig {
    let root = FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString, isDirectory: true)
    let audio = AudioSessionProbe()
    let log = EventLog()
    let manager = PlaybackManager(storageRoot: root, activateAudioSession: audio.claim) { log.record($0) }
    return EngineRig(manager: manager, root: root, audio: audio, log: log)
}

/// Posts what the system posts when the audio session is interrupted or its route changes.
@MainActor
func postInterruption(
    _ type: AVAudioSession.InterruptionType, options: AVAudioSession.InterruptionOptions = [],
) {
    NotificationCenter.default.post(
        name: AVAudioSession.interruptionNotification,
        object: AVAudioSession.sharedInstance(),
        userInfo: [
            AVAudioSessionInterruptionTypeKey: type.rawValue,
            AVAudioSessionInterruptionOptionKey: options.rawValue,
        ],
    )
}

@MainActor
func postRouteChange(_ reason: AVAudioSession.RouteChangeReason) {
    NotificationCenter.default.post(
        name: AVAudioSession.routeChangeNotification,
        object: AVAudioSession.sharedInstance(),
        userInfo: [AVAudioSessionRouteChangeReasonKey: reason.rawValue],
    )
}

/// Polls (on the main actor) until `condition` holds, failing the test if it doesn't within
/// `timeout` seconds. Engine news arrives asynchronously.
@MainActor
func waitUntil(timeout: Double = 5, _ condition: @MainActor () -> Bool) async throws {
    let deadline = Date().addingTimeInterval(timeout)
    while !condition() {
        guard Date() < deadline else {
            Issue.record("timed out waiting for the engine")
            return
        }
        try await Task.sleep(for: .milliseconds(20))
    }
}

@Suite @MainActor struct PlaybackManagerTests {
    @Test func loadingAPlayableFileLoadsIt() throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()

        let result = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )

        #expect(result == .ok)
        #expect(rig.manager.hasLoadedItem)
        #expect(rig.manager.currentSession == 1)
    }

    @Test func aFailedLoadLeavesNothingLoadedNotTheOldItem() throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()
        #expect(
            rig.manager.perform(
                .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: false),
            ) == .ok,
        )

        // The next episode's file is gone: the engine must not keep the previous one.
        let result = rig.manager.perform(
            .load(session: 2, media: .local(localPath: "Downloads/missing.mp3"), startSecs: 0, autoplay: true),
        )

        #expect(result == .mediaUnusable("The downloaded file is missing"))
        #expect(!rig.manager.hasLoadedItem)
        #expect(rig.manager.currentSession == nil)
    }

    @Test func anUnusableStreamURLAlsoUnloadsThePreviousItem() throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()
        _ = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )

        let result = rig.manager.perform(
            .load(session: 2, media: .stream(url: ""), startSecs: 0, autoplay: true),
        )

        #expect(result == .mediaUnusable("Invalid episode URL"))
        #expect(!rig.manager.hasLoadedItem)
        #expect(rig.manager.currentSession == nil)
    }

    @Test func aFailedLoadWithNothingLoadedIsHarmless() {
        let rig = makeEngineRig()

        let result = rig.manager.perform(
            .load(session: 1, media: .local(localPath: "Downloads/missing.mp3"), startSecs: 0, autoplay: false),
        )

        #expect(result == .mediaUnusable("The downloaded file is missing"))
        #expect(!rig.manager.hasLoadedItem)
    }

    // MARK: - Audio focus

    @Test func aPausedLoadDoesNotClaimTheAudioSession() throws {
        // A source swap that finishes while the listener has paused (and moved on to another
        // app's music) must not cut that audio off.
        let rig = makeEngineRig()
        let path = try rig.installAudio()

        let result = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )

        #expect(result == .ok)
        #expect(rig.audio.claims == 0)
    }

    @Test func aLoadThatWillPlayClaimsTheAudioSession() throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()

        let result = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: true),
        )

        #expect(result == .ok)
        #expect(rig.audio.claims == 1)
    }

    @Test func playingAfterAPausedLoadClaimsTheAudioSessionThen() throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()
        _ = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )
        #expect(rig.audio.claims == 0)

        let result = rig.manager.perform(.play)

        #expect(result == .ok)
        #expect(rig.audio.claims == 1)
    }

    @Test func pausingAndSeekingDoNotClaimTheAudioSession() throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()
        _ = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )

        _ = rig.manager.perform(.seek(secs: 0))
        _ = rig.manager.perform(.pause)

        #expect(rig.audio.claims == 0)
    }

    @Test func aRefusedSessionFailsAPlayingLoadAndUnloadsTheItem() throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()
        rig.audio.refuse = true

        let result = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: true),
        )

        #expect(result == .error("Couldn't start audio: audio is busy"))
        #expect(!rig.manager.hasLoadedItem)
    }

    @Test func aRefusedSessionFailsPlayButKeepsTheLoadedItem() throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()
        _ = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )
        rig.audio.refuse = true

        let result = rig.manager.perform(.play)

        #expect(result == .error("Couldn't start audio: audio is busy"))
        // Nothing is wrong with the item; a later play (once audio is free) can use it.
        #expect(rig.manager.hasLoadedItem)
    }

    @Test func aSuccessfulLoadAfterAFailedOneWorks() throws {
        // The core falls back to another source right after a failed load.
        let rig = makeEngineRig()
        let path = try rig.installAudio()
        _ = rig.manager.perform(
            .load(session: 1, media: .local(localPath: "Downloads/missing.mp3"), startSecs: 0, autoplay: false),
        )

        let result = rig.manager.perform(
            .load(session: 2, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )

        #expect(result == .ok)
        #expect(rig.manager.hasLoadedItem)
        #expect(rig.manager.currentSession == 2)
    }

    // MARK: - System notifications → core events

    @Test func anInterruptionBeginningIsReportedAsResumable() {
        let rig = makeEngineRig()
        postInterruption(.began)
        #expect(rig.log.events == [.interrupted(resumable: true)])
    }

    @Test func anInterruptionEndingReportsWhetherTheSystemAllowsResuming() {
        let rig = makeEngineRig()
        postInterruption(.ended, options: [.shouldResume])
        postInterruption(.ended)
        #expect(rig.log.events == [
            .interruptionEnded(shouldResume: true),
            .interruptionEnded(shouldResume: false),
        ])
    }

    @Test func unpluggingHeadphonesIsAnInterruptionThatNeverResumes() {
        let rig = makeEngineRig()
        postRouteChange(.oldDeviceUnavailable)
        #expect(rig.log.events == [.interrupted(resumable: false)])
    }

    @Test func otherRouteChangesAreNotReported() {
        let rig = makeEngineRig()
        postRouteChange(.newDeviceAvailable)
        postRouteChange(.categoryChange)
        postRouteChange(.override)
        #expect(rig.log.events.isEmpty)
    }

    @Test func notificationsFromOtherObjectsAreIgnored() {
        let rig = makeEngineRig()
        NotificationCenter.default.post(
            name: AVAudioSession.interruptionNotification,
            object: NSObject(),
            userInfo: [AVAudioSessionInterruptionTypeKey: AVAudioSession.InterruptionType.began.rawValue],
        )
        #expect(rig.log.events.isEmpty)
    }

    // MARK: - Session tagging

    @Test func engineNewsCarriesTheSessionOfTheLoadThatProducedIt() async throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()

        _ = rig.manager.perform(
            .load(session: 7, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )
        try await waitUntil { rig.log.events.contains(.playerDuration(session: 7, durationSecs: 1)) }

        // A newer load replaces it; its news must carry the new session.
        _ = rig.manager.perform(
            .load(session: 8, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )
        try await waitUntil { rig.log.events.contains(.playerDuration(session: 8, durationSecs: 1)) }
    }

    @Test func aLoadReplacedBeforeItIsReadyReportsNothingForTheOldSession() async throws {
        let rig = makeEngineRig()
        let path = try rig.installAudio()

        // Replaced in the same turn, before the first item can report anything.
        _ = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )
        _ = rig.manager.perform(
            .load(session: 2, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )
        try await waitUntil { rig.log.events.contains(.playerDuration(session: 2, durationSecs: 1)) }

        let stale = rig.log.events.filter {
            if case let .playerDuration(session, _) = $0 { return session == 1 }
            return false
        }
        #expect(stale.isEmpty)
    }
}
