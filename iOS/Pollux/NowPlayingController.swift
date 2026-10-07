import App
import Foundation
import MediaPlayer
import UIKit

/// Lock-screen, Control Center, headphone and car integration. Remote commands are
/// forwarded to the core as the same events the in-app controls send, so the lock
/// screen and the player can never disagree; the Now Playing card is rebuilt from the
/// core's `PlayerView` on every render.
@MainActor
final class NowPlayingController {
    private let send: (Event) -> Void
    private var artworkURL: String?
    private var artwork: MPMediaItemArtwork?
    /// What the card currently shows, so an unchanged state isn't rewritten (see
    /// `NowPlayingPolicy`). Nil while the card is empty.
    private var published: NowPlayingPublication?
    /// The latest state seen, published or not, for republishing when artwork arrives.
    private var latest: PlayerView?

    init(send: @escaping (Event) -> Void) {
        self.send = send
        registerCommands()
    }

    /// Called on every render; writes the card only when `NowPlayingPolicy` says it is out
    /// of date. `now` is injectable for tests.
    func update(_ player: PlayerView?, now: TimeInterval = ProcessInfo.processInfo.systemUptime) {
        guard let player else {
            clear()
            return
        }
        latest = player
        loadArtworkIfNeeded(player.artworkUrl)
        guard NowPlayingPolicy.needsPublishing(player, since: published, now: now) else { return }
        publish(player, now: now)
    }

    private func clear() {
        latest = nil
        // Renders with nothing playing are frequent; only touch the card if it has content.
        guard published != nil else { return }
        published = nil
        let center = MPNowPlayingInfoCenter.default()
        center.nowPlayingInfo = nil
        center.playbackState = .stopped
    }

    private func publish(_ player: PlayerView, now: TimeInterval) {
        let center = MPNowPlayingInfoCenter.default()
        var info: [String: Any] = [
            MPMediaItemPropertyTitle: player.episodeTitle,
            MPMediaItemPropertyArtist: player.feedTitle,
            MPMediaItemPropertyAlbumTitle: player.feedTitle,
            MPNowPlayingInfoPropertyElapsedPlaybackTime: Double(player.positionSecs),
            MPNowPlayingInfoPropertyPlaybackRate: player.isPlaying ? 1.0 : 0.0,
            MPNowPlayingInfoPropertyDefaultPlaybackRate: 1.0,
        ]
        if let duration = player.durationSecs {
            info[MPMediaItemPropertyPlaybackDuration] = Double(duration)
        }
        if let artwork {
            info[MPMediaItemPropertyArtwork] = artwork
        }
        center.nowPlayingInfo = info
        center.playbackState = player.isPlaying ? .playing : .paused

        let snapshot = NowPlayingSnapshot(player)
        let skipChanged = published.map {
            $0.snapshot.skipForwardSecs != snapshot.skipForwardSecs
                || $0.snapshot.skipBackSecs != snapshot.skipBackSecs
        } ?? true
        if skipChanged {
            updateSkipIntervals(forward: player.skipForwardSecs, back: player.skipBackSecs)
        }
        published = NowPlayingPublication(
            snapshot: snapshot, elapsed: Double(player.positionSecs), publishedAt: now,
        )
    }

    // MARK: - Remote commands

    private func registerCommands() {
        let commands = MPRemoteCommandCenter.shared()
        // Podcast convention: skip buttons stand in for next/previous track.
        commands.nextTrackCommand.isEnabled = false
        commands.previousTrackCommand.isEnabled = false

        add(commands.playCommand, .play)
        add(commands.pauseCommand, .pause)
        add(commands.togglePlayPauseCommand, .togglePlay)
        add(commands.skipForwardCommand, .skipForward)
        add(commands.skipBackwardCommand, .skipBack)

        commands.changePlaybackPositionCommand.isEnabled = true
        commands.changePlaybackPositionCommand.addTarget { [weak self] event in
            guard let event = event as? MPChangePlaybackPositionCommandEvent else {
                return .commandFailed
            }
            // A car or Bluetooth device can send nonsense (NaN, a huge value); refuse it
            // rather than trap converting it.
            guard let secs = PlayerFormatting.wholeSeconds(event.positionTime) else {
                return .commandFailed
            }
            Task { @MainActor in self?.send(.seekTo(secs)) }
            return .success
        }
    }

    private func add(_ command: MPRemoteCommand, _ event: Event) {
        command.isEnabled = true
        command.addTarget { [weak self] _ in
            Task { @MainActor in self?.send(event) }
            return .success
        }
    }

    private func updateSkipIntervals(forward: UInt32, back: UInt32) {
        let commands = MPRemoteCommandCenter.shared()
        commands.skipForwardCommand.preferredIntervals = [NSNumber(value: forward)]
        commands.skipBackwardCommand.preferredIntervals = [NSNumber(value: back)]
    }

    // MARK: - Artwork

    /// Fetches the card's artwork once per URL; the card shows without it meanwhile.
    private func loadArtworkIfNeeded(_ urlString: String?) {
        guard urlString != artworkURL else { return }
        artworkURL = urlString
        artwork = nil
        guard let urlString, let url = URL(string: urlString) else { return }
        Task { [weak self] in
            guard let (data, _) = try? await URLSession.shared.data(from: url),
                  let image = UIImage(data: data) else { return }
            self?.applyArtwork(image, for: urlString)
        }
    }

    private func applyArtwork(_ image: UIImage, for urlString: String) {
        // The episode may have changed while the image downloaded.
        guard artworkURL == urlString else { return }
        artwork = MPMediaItemArtwork(boundsSize: image.size) { _ in image }
        // Republish from the latest state rather than patching the card in place: the card
        // holds an elapsed time from when it was last written, which may be many seconds
        // old now, and rewriting it would set the system's clock back to that.
        if let latest {
            publish(latest, now: ProcessInfo.processInfo.systemUptime)
        }
    }
}
