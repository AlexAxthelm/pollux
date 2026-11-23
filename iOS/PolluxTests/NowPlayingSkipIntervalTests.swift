import App
import Foundation
import MediaPlayer
@testable import Pollux
import Testing

// The lock screen shows the skip buttons from the remote commands' preferred intervals, so
// the controller has to keep them in step with the player's skip settings (settings-cascade).

private func player(skipForward: UInt32, skipBack: UInt32, playing: Bool = true) -> PlayerView {
    PlayerView(
        episodeId: "ep-1", episodeTitle: "Episode", feedTitle: "Feed",
        source: .subscription(id: "sub-1"), sourceTitle: "Feed", artworkUrl: nil,
        positionSecs: 100, durationSecs: 3600, isPlaying: playing, isStreaming: false,
        skipForwardSecs: skipForward, skipBackSecs: skipBack, description: nil,
        descriptionText: nil, error: nil,
    )
}

private func intervals() -> (forward: [Double], back: [Double]) {
    let commands = MPRemoteCommandCenter.shared()
    return (
        commands.skipForwardCommand.preferredIntervals.map(\.doubleValue),
        commands.skipBackwardCommand.preferredIntervals.map(\.doubleValue),
    )
}

/// The remote command center is one per process, so these run one at a time.
@Suite(.serialized) @MainActor struct NowPlayingSkipIntervalTests {
    @Test func `the first card sets the lock screen skip intervals`() {
        let controller = NowPlayingController { _ in }
        controller.update(player(skipForward: 60, skipBack: 10), now: 1000)
        #expect(intervals().forward == [60])
        #expect(intervals().back == [10])
    }

    @Test func `a changed skip setting reaches the lock screen while playing`() {
        let controller = NowPlayingController { _ in }
        controller.update(player(skipForward: 30, skipBack: 15), now: 1000)
        controller.update(player(skipForward: 45, skipBack: 5), now: 1001)
        #expect(intervals().forward == [45])
        #expect(intervals().back == [5])
    }

    @Test func `a changed skip setting reaches the lock screen while paused`() {
        let controller = NowPlayingController { _ in }
        controller.update(player(skipForward: 30, skipBack: 15, playing: false), now: 1000)
        // Nothing else about the card changes: the settings page alone moves the intervals.
        controller.update(player(skipForward: 90, skipBack: 10, playing: false), now: 1500)
        #expect(intervals().forward == [90])
        #expect(intervals().back == [10])
    }
}
