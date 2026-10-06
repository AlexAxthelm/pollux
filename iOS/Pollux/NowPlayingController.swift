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

    init(send: @escaping (Event) -> Void) {
        self.send = send
        registerCommands()
    }

    func update(_ player: PlayerView?) {
        let center = MPNowPlayingInfoCenter.default()
        guard let player else {
            center.nowPlayingInfo = nil
            center.playbackState = .stopped
            return
        }
        loadArtworkIfNeeded(player.artworkUrl)

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
        updateSkipIntervals(forward: player.skipForwardSecs, back: player.skipBackSecs)
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
            let secs = UInt32(max(0, event.positionTime))
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
        let artwork = MPMediaItemArtwork(boundsSize: image.size) { _ in image }
        self.artwork = artwork
        var info = MPNowPlayingInfoCenter.default().nowPlayingInfo
        info?[MPMediaItemPropertyArtwork] = artwork
        MPNowPlayingInfoCenter.default().nowPlayingInfo = info
    }
}
