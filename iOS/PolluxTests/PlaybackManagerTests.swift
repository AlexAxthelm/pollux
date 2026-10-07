import App
import Foundation
import Testing

@testable import Pollux

// These drive the real `PlaybackManager` (and so a real `AVPlayer`) against files in a
// temporary storage root. Nothing is played: loads are paused (`autoplay: false`), which is
// enough to see what the engine has loaded.

/// One second of silence as a minimal PCM WAV file, the smallest valid audio `AVPlayer`
/// will open.
private func silentWAV() -> Data {
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
private final class AudioSessionProbe: @unchecked Sendable {
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

@MainActor
private struct Rig {
    let manager: PlaybackManager
    let root: URL
    let audio: AudioSessionProbe

    /// A relative path (as the core stores them) to a real, playable file under the root.
    func installAudio(named name: String = "silence.wav") throws -> String {
        let directory = root.appendingPathComponent("Downloads", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try silentWAV().write(to: directory.appendingPathComponent(name))
        return "Downloads/\(name)"
    }
}

@MainActor
private func makeRig() -> Rig {
    let root = FileManager.default.temporaryDirectory
        .appendingPathComponent(UUID().uuidString, isDirectory: true)
    let audio = AudioSessionProbe()
    let manager = PlaybackManager(storageRoot: root, activateAudioSession: audio.claim) { _ in }
    return Rig(manager: manager, root: root, audio: audio)
}

@Suite @MainActor struct PlaybackManagerTests {
    @Test func loadingAPlayableFileLoadsIt() throws {
        let rig = makeRig()
        let path = try rig.installAudio()

        let result = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )

        #expect(result == .ok)
        #expect(rig.manager.hasLoadedItem)
        #expect(rig.manager.currentSession == 1)
    }

    @Test func aFailedLoadLeavesNothingLoadedNotTheOldItem() throws {
        let rig = makeRig()
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
        let rig = makeRig()
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
        let rig = makeRig()

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
        let rig = makeRig()
        let path = try rig.installAudio()

        let result = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )

        #expect(result == .ok)
        #expect(rig.audio.claims == 0)
    }

    @Test func aLoadThatWillPlayClaimsTheAudioSession() throws {
        let rig = makeRig()
        let path = try rig.installAudio()

        let result = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: true),
        )

        #expect(result == .ok)
        #expect(rig.audio.claims == 1)
    }

    @Test func playingAfterAPausedLoadClaimsTheAudioSessionThen() throws {
        let rig = makeRig()
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
        let rig = makeRig()
        let path = try rig.installAudio()
        _ = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: false),
        )

        _ = rig.manager.perform(.seek(secs: 0))
        _ = rig.manager.perform(.pause)

        #expect(rig.audio.claims == 0)
    }

    @Test func aRefusedSessionFailsAPlayingLoadAndUnloadsTheItem() throws {
        let rig = makeRig()
        let path = try rig.installAudio()
        rig.audio.refuse = true

        let result = rig.manager.perform(
            .load(session: 1, media: .local(localPath: path), startSecs: 0, autoplay: true),
        )

        #expect(result == .error("Couldn't start audio: audio is busy"))
        #expect(!rig.manager.hasLoadedItem)
    }

    @Test func aRefusedSessionFailsPlayButKeepsTheLoadedItem() throws {
        let rig = makeRig()
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
        let rig = makeRig()
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
}
