import App
import AVFoundation
import Foundation

/// The audio engine for the `Player` capability: owns the `AVPlayer`, the audio
/// session and its interruptions, and reports engine news back to the core as events.
/// It carries out what the core asks and makes no playback decisions of its own — when
/// to rewind, how far to skip, and what counts as played all live in the core.
@MainActor
final class PlaybackManager {
    private let storageRoot: URL
    private let player = AVPlayer()
    /// Core events (ticks, end of file, interruptions…) go out through this.
    private let send: (Event) -> Void

    /// The load the engine is on. Everything reported to the core carries it, so the
    /// core can drop news from an item it has since replaced.
    private(set) var currentSession: UInt32?
    /// Seeks still in flight. A streaming seek takes a moment, and the engine keeps
    /// reporting the old position until it lands; those ticks would snap the UI back.
    private var pendingSeeks = 0
    private var timeObserver: Any?
    private var statusObservation: NSKeyValueObservation?
    private var notificationTokens: [NSObjectProtocol] = []

    /// Claims the audio session for playback. Injectable so tests can see when it is
    /// claimed: doing so takes audio focus from other apps, so it must only happen when
    /// audio is about to play.
    private let activateAudioSession: () throws -> Void

    init(
        storageRoot: URL,
        activateAudioSession: @escaping () throws -> Void = PlaybackManager.activatePlaybackSession,
        send: @escaping (Event) -> Void,
    ) {
        self.storageRoot = storageRoot
        self.activateAudioSession = activateAudioSession
        self.send = send
        observeTime()
        observeSession()
    }

    nonisolated static func activatePlaybackSession() throws {
        let session = AVAudioSession.sharedInstance()
        try session.setCategory(.playback, mode: .spokenAudio)
        try session.setActive(true)
    }

    /// Whether the engine has an item loaded. Exposed so tests can tell a failed load that
    /// left the old item playing from one that unloaded it.
    var hasLoadedItem: Bool {
        player.currentItem != nil
    }

    // MARK: - Operations

    /// Carries out one core operation. Synchronous: AVPlayer queues its own work.
    func perform(_ operation: PlayerOperation) -> PlayerResult {
        switch operation {
        case let .load(session, media, startSecs, autoplay):
            return load(session: session, media: media, startSecs: startSecs, autoplay: autoplay)
        case .play:
            // The session may not be active: a paused load (a source swap, a scrub of a
            // restored episode) deliberately doesn't claim it.
            if let failure = claimAudioSession() {
                return failure
            }
            player.play()
        case .pause:
            player.pause()
        case let .seek(secs):
            seek(to: secs)
        case .stop:
            stop()
        }
        return .ok
    }

    /// Replaces the current item with `media`. If that can't be done the engine is left with
    /// nothing loaded rather than with whatever was playing before: the core has already
    /// moved on to this item (paused, with the error), so audio from the old one would play
    /// on under a UI that says nothing is playing, with its news dropped as stale.
    private func load(
        session: UInt32, media: MediaSource, startSecs: UInt32, autoplay: Bool,
    ) -> PlayerResult {
        let result = startLoading(session: session, media: media, startSecs: startSecs, autoplay: autoplay)
        switch result {
        case .error, .mediaUnusable:
            unloadCurrentItem()
        case .ok:
            break
        }
        return result
    }

    private func startLoading(
        session: UInt32, media: MediaSource, startSecs: UInt32, autoplay: Bool,
    ) -> PlayerResult {
        let url: URL
        switch media {
        case let .stream(urlString):
            guard let parsed = URL(string: urlString) else {
                return .mediaUnusable("Invalid episode URL")
            }
            url = parsed
        case let .local(localPath):
            url = DownloadManager.absoluteURL(for: localPath, root: storageRoot)
            guard FileManager.default.fileExists(atPath: url.path) else {
                return .mediaUnusable("The downloaded file is missing")
            }
        }

        // A paused load must not take audio focus: the listener may have switched to another
        // app's music, and a source swap finishing in the background would cut it off.
        if autoplay, let failure = claimAudioSession() {
            return failure
        }

        currentSession = session
        let item = AVPlayerItem(url: url)
        observe(item, session: session)
        player.replaceCurrentItem(with: item)
        // Exact seek: a resume that lands a few seconds off would defeat the rewind.
        seek(to: startSecs)
        if autoplay {
            player.play()
        }
        return .ok
    }

    /// Claims the audio session, or returns the error to report.
    private func claimAudioSession() -> PlayerResult? {
        do {
            try activateAudioSession()
            return nil
        } catch {
            return .error("Couldn't start audio: \(error.localizedDescription)")
        }
    }

    private func seek(to secs: UInt32) {
        let target = CMTime(seconds: Double(secs), preferredTimescale: 600)
        pendingSeeks += 1
        player.seek(to: target, toleranceBefore: .zero, toleranceAfter: .zero) { [weak self] _ in
            // Also fires (unfinished) for a seek superseded by a newer one.
            MainActor.assumeIsolated { self?.pendingSeeks -= 1 }
        }
    }

    private func stop() {
        unloadCurrentItem()
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
    }

    /// Drops the current item and stops reporting for it. Leaves the audio session alone:
    /// after a failed load the core may immediately load another source (a missing
    /// download falls back to streaming), and releasing the session in between would
    /// briefly hand audio back to other apps.
    private func unloadCurrentItem() {
        player.pause()
        player.replaceCurrentItem(with: nil)
        statusObservation = nil
        currentSession = nil
    }

    // MARK: - Engine → core

    private func observe(_ item: AVPlayerItem, session: UInt32) {
        statusObservation = item.observe(\.status, options: [.new]) { [weak self] item, _ in
            let status = item.status
            let duration = item.duration.seconds
            let message = item.error?.localizedDescription
            Task { @MainActor [weak self] in
                guard let self, currentSession == session else { return }
                switch status {
                case .readyToPlay:
                    if duration > 0, let secs = PlayerFormatting.wholeSeconds(duration) {
                        send(.playerDuration(session: session, durationSecs: secs))
                    }
                case .failed:
                    send(.playerMediaUnusable(session: session, message: message ?? "Playback failed"))
                default:
                    break
                }
            }
        }
        notificationTokens.forEach(NotificationCenter.default.removeObserver)
        notificationTokens = [
            NotificationCenter.default.addObserver(
                forName: AVPlayerItem.didPlayToEndTimeNotification, object: item, queue: .main,
            ) { [weak self] _ in
                MainActor.assumeIsolated { self?.send(.playerEnded(session: session)) }
            },
            NotificationCenter.default.addObserver(
                forName: AVPlayerItem.failedToPlayToEndTimeNotification, object: item, queue: .main,
            ) { [weak self] note in
                let error = note.userInfo?[AVPlayerItemFailedToPlayToEndTimeErrorKey] as? Error
                let message = error?.localizedDescription ?? "Playback stopped unexpectedly"
                MainActor.assumeIsolated {
                    self?.send(.playerFailed(session: session, message: message))
                }
            },
        ]
    }

    /// Reports position about once a second while playing.
    private func observeTime() {
        let interval = CMTime(seconds: 1, preferredTimescale: 600)
        timeObserver = player.addPeriodicTimeObserver(forInterval: interval, queue: .main) { [weak self] time in
            let seconds = time.seconds
            MainActor.assumeIsolated { self?.reportTick(seconds: seconds) }
        }
    }

    private func reportTick(seconds: Double) {
        guard let session = currentSession,
              pendingSeeks == 0,
              player.timeControlStatus == .playing,
              seconds >= 0,
              let secs = PlayerFormatting.wholeSeconds(seconds) else { return }
        send(.playerTick(session: session, positionSecs: secs))
    }

    /// Phone calls, Siri and unplugged headphones pause playback; the core is told so
    /// its state and the persisted position stay right. The core decides whether an
    /// interruption's end resumes playback: only if the interruption is what paused it
    /// and the system says it may (it also says so after interruptions that found
    /// playback already paused). Unplugging headphones is never resumed.
    private func observeSession() {
        let center = NotificationCenter.default
        let session = AVAudioSession.sharedInstance()
        center.addObserver(
            forName: AVAudioSession.interruptionNotification, object: session, queue: .main,
        ) { [weak self] note in
            let info = note.userInfo
            let type = (info?[AVAudioSessionInterruptionTypeKey] as? UInt)
                .flatMap(AVAudioSession.InterruptionType.init(rawValue:))
            let options = (info?[AVAudioSessionInterruptionOptionKey] as? UInt)
                .map(AVAudioSession.InterruptionOptions.init(rawValue:))
            MainActor.assumeIsolated {
                switch type {
                case .began:
                    self?.send(.interrupted(resumable: true))
                case .ended:
                    let shouldResume = options?.contains(.shouldResume) == true
                    self?.send(.interruptionEnded(shouldResume: shouldResume))
                default:
                    break
                }
            }
        }
        center.addObserver(
            forName: AVAudioSession.routeChangeNotification, object: session, queue: .main,
        ) { [weak self] note in
            let reason = (note.userInfo?[AVAudioSessionRouteChangeReasonKey] as? UInt)
                .flatMap(AVAudioSession.RouteChangeReason.init(rawValue:))
            MainActor.assumeIsolated {
                if reason == .oldDeviceUnavailable {
                    // No "ended" notification follows a route change, so never resumable.
                    self?.send(.interrupted(resumable: false))
                }
            }
        }
    }
}
