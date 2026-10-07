import App
import Foundation
import Testing

@testable import Pollux

// The lock-screen card runs its own clock from an elapsed time and a rate. Rewriting it
// resets that clock, so it should be rewritten only when the card's content changes or the
// playhead leaves the clock's course, not on every render.

private func player(
    position: UInt32 = 100,
    playing: Bool = true,
    title: String = "Episode",
    duration: UInt32? = 3600,
    artwork: String? = "https://example.com/a.jpg",
    skipForward: UInt32 = 30,
    skipBack: UInt32 = 15,
    error: String? = nil,
) -> PlayerView {
    PlayerView(
        episodeId: "ep-1", episodeTitle: title, feedTitle: "Feed",
        source: .subscription(id: "sub-1"), sourceTitle: "Feed", artworkUrl: artwork,
        positionSecs: position, durationSecs: duration, isPlaying: playing, isStreaming: false,
        skipForwardSecs: skipForward, skipBackSecs: skipBack, description: nil,
        descriptionText: nil, error: error,
    )
}

/// A card as last written for `player` at time `at`.
private func published(_ player: PlayerView, at time: TimeInterval = 1000) -> NowPlayingPublication {
    NowPlayingPublication(
        snapshot: NowPlayingSnapshot(player), elapsed: Double(player.positionSecs), publishedAt: time,
    )
}

private func needsPublishing(_ current: PlayerView, since last: NowPlayingPublication?, at time: TimeInterval) -> Bool {
    NowPlayingPolicy.needsPublishing(current, since: last, now: time)
}

@Suite struct NowPlayingPolicyTests {
    @Test func theFirstStateIsPublished() {
        #expect(needsPublishing(player(), since: nil, at: 1000))
    }

    @Test func ordinaryPlaybackNeverRewritesTheCard() {
        // A tick a second for ten minutes: the playhead keeps pace with the card's clock.
        let first = player(position: 100)
        let last = published(first, at: 1000)

        for second in 1 ... 600 {
            let tick = player(position: 100 + UInt32(second))
            #expect(!needsPublishing(tick, since: last, at: 1000 + TimeInterval(second)))
        }
    }

    @Test func aWholeSecondOfJitterIsTolerated() {
        // The core reports truncated whole seconds, so a tick can lag the clock by up to one.
        let last = published(player(position: 100), at: 1000)
        #expect(!needsPublishing(player(position: 101), since: last, at: 1002.9))
        #expect(!needsPublishing(player(position: 103), since: last, at: 1002.1))
    }

    @Test func pausingAndResumingRewriteTheCard() {
        let playing = published(player(playing: true))
        #expect(needsPublishing(player(playing: false), since: playing, at: 1005))

        let paused = published(player(playing: false))
        #expect(needsPublishing(player(playing: true), since: paused, at: 1005))
    }

    @Test func aSkipOrSeekRewritesTheCard() {
        let last = published(player(position: 100), at: 1000)
        // +30 skip a second later.
        #expect(needsPublishing(player(position: 131), since: last, at: 1001))
        // -15 skip a second later.
        #expect(needsPublishing(player(position: 86), since: last, at: 1001))
    }

    @Test func aStallRewritesTheCardOnceTheGapIsNoticeable() {
        // Buffering: the player says it is playing but the position stands still, so the
        // card's clock runs ahead of it.
        let last = published(player(position: 100), at: 1000)
        #expect(!needsPublishing(player(position: 100), since: last, at: 1001.5))
        #expect(needsPublishing(player(position: 100), since: last, at: 1005))
    }

    @Test func aPausedCardIsLeftAloneHoweverLongItStays() {
        let last = published(player(position: 100, playing: false), at: 1000)
        #expect(!needsPublishing(player(position: 100, playing: false), since: last, at: 50000))
    }

    @Test func scrubbingWhilePausedRewritesTheCard() {
        let last = published(player(position: 100, playing: false), at: 1000)
        #expect(needsPublishing(player(position: 400, playing: false), since: last, at: 1010))
    }

    @Test func contentChangesRewriteTheCard() {
        let last = published(player())
        // The real duration arrives once the engine has loaded the file.
        #expect(needsPublishing(player(duration: 3639), since: last, at: 1001))
        #expect(needsPublishing(player(title: "Another"), since: last, at: 1001))
        #expect(needsPublishing(player(artwork: nil), since: last, at: 1001))
        #expect(needsPublishing(player(skipForward: 45), since: last, at: 1001))
        #expect(needsPublishing(player(skipBack: 10), since: last, at: 1001))
    }

    @Test func thingsTheCardDoesNotShowAreIgnored() {
        let last = published(player())
        #expect(!needsPublishing(player(position: 101, error: "Couldn't start audio"), since: last, at: 1001))
    }
}
