import App
import Foundation

/// What the lock-screen card shows, apart from where the playhead is. When any of these
/// changes the card must be rewritten.
struct NowPlayingSnapshot: Equatable {
    let episodeId: String
    let title: String
    let feedTitle: String
    let artworkUrl: String?
    let duration: UInt32?
    let isPlaying: Bool
    let skipForwardSecs: UInt32
    let skipBackSecs: UInt32

    init(_ player: PlayerView) {
        episodeId = player.episodeId
        title = player.episodeTitle
        feedTitle = player.feedTitle
        artworkUrl = player.artworkUrl
        duration = player.durationSecs
        isPlaying = player.isPlaying
        skipForwardSecs = player.skipForwardSecs
        skipBackSecs = player.skipBackSecs
    }
}

/// What was last written to the card, and when. The system runs the card's clock itself
/// from an elapsed time and a rate, so for as long as the playhead keeps pace with that
/// clock there is nothing to tell it.
struct NowPlayingPublication {
    let snapshot: NowPlayingSnapshot
    let elapsed: Double
    /// A monotonic timestamp (`ProcessInfo.systemUptime`) of when it was written.
    let publishedAt: TimeInterval
}

enum NowPlayingPolicy {
    /// How far the playhead may stray from the card's own clock before the card is
    /// corrected. The core reports whole seconds, so ordinary playback is always within one
    /// second of the clock; this leaves room for that and for timing jitter.
    static let driftTolerance: Double = 2

    /// Whether the card needs rewriting for `player`.
    ///
    /// Rewriting it resets the system's clock, so doing it on every render (every second
    /// of playback, and on every download or refresh update besides) makes the card's
    /// scrubber jitter and does needless work. It is rewritten when what it shows changes,
    /// or when the playhead has left the clock's course: a seek or skip, or playback that
    /// stalled (buffering) or caught up.
    static func needsPublishing(
        _ player: PlayerView, since last: NowPlayingPublication?, now: TimeInterval,
    ) -> Bool {
        guard let last else { return true }
        if NowPlayingSnapshot(player) != last.snapshot {
            return true
        }
        let expected = last.snapshot.isPlaying ? last.elapsed + (now - last.publishedAt) : last.elapsed
        return abs(Double(player.positionSecs) - expected) > driftTolerance
    }
}
