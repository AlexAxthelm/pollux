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
    private var currentSession: UInt32?
    /// Seeks still in flight. A streaming seek takes a moment, and the engine keeps
    /// reporting the old position until it lands; those ticks would snap the UI back.
    private var pendingSeeks = 0
    private var timeObserver: Any?
    private var statusObservation: NSKeyValueObservation?
    private var notificationTokens: [NSObjectProtocol] = []

    init(storageRoot: URL, send: @escaping (Event) -> Void) {
        self.storageRoot = storageRoot
        self.send = send
        observeTime()
        observeSession()
    }

    // MARK: - Operations

    /// Carries out one core operation. Synchronous: AVPlayer queues its own work.
    func perform(_ operation: PlayerOperation) -> PlayerResult {
        switch operation {
        case let .load(session, media, startSecs, autoplay):
            return load(session: session, media: media, startSecs: startSecs, autoplay: autoplay)
        case .play:
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

    private func load(
        session: UInt32, media: MediaSource, startSecs: UInt32, autoplay: Bool,
    ) -> PlayerResult {
        let url: URL
        switch media {
        case let .stream(urlString):
            guard let parsed = URL(string: urlString) else {
                return .error("Invalid episode URL")
            }
            url = parsed
        case let .local(localPath):
            url = DownloadManager.absoluteURL(for: localPath, root: storageRoot)
            guard FileManager.default.fileExists(atPath: url.path) else {
                return .error("The downloaded file is missing")
            }
        }

        do {
            let session = AVAudioSession.sharedInstance()
            try session.setCategory(.playback, mode: .spokenAudio)
            try session.setActive(true)
        } catch {
            return .error("Couldn't start audio: \(error.localizedDescription)")
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

    private func seek(to secs: UInt32) {
        let target = CMTime(seconds: Double(secs), preferredTimescale: 600)
        pendingSeeks += 1
        player.seek(to: target, toleranceBefore: .zero, toleranceAfter: .zero) { [weak self] _ in
            // Also fires (unfinished) for a seek superseded by a newer one.
            MainActor.assumeIsolated { self?.pendingSeeks -= 1 }
        }
    }

    private func stop() {
        player.pause()
        player.replaceCurrentItem(with: nil)
        statusObservation = nil
        currentSession = nil
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
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
                    if duration.isFinite, duration > 0 {
                        send(.playerDuration(session: session, durationSecs: UInt32(duration)))
                    }
                case .failed:
                    send(.playerFailed(session: session, message: message ?? "Playback failed"))
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
              seconds.isFinite, seconds >= 0 else { return }
        send(.playerTick(session: session, positionSecs: UInt32(seconds)))
    }

    /// Phone calls, Siri and unplugged headphones pause playback; the core is told so
    /// its state and the persisted position stay right. Playback resumes only when the
    /// system says it should (e.g. after a call), never after unplugging.
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
                    self?.send(.interrupted)
                case .ended where options?.contains(.shouldResume) == true:
                    self?.send(.play)
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
                    self?.send(.interrupted)
                }
            }
        }
    }
}
