import Foundation
import Testing

@testable import Pollux

@Suite("EpisodeFormatting")
struct EpisodeFormattingTests {

    // MARK: Duration

    @Test func nilDurationIsNil() {
        #expect(EpisodeFormatting.formatDuration(nil) == nil)
    }

    @Test func zeroDurationIsNil() {
        #expect(EpisodeFormatting.formatDuration(0) == nil)
    }

    @Test func subMinuteDurationShowsSeconds() {
        #expect(EpisodeFormatting.formatDuration(30) == "30s")
    }

    @Test func minutesOnlyDurationOmitsSeconds() {
        #expect(EpisodeFormatting.formatDuration(45 * 60) == "45m")
    }

    @Test func hoursAndMinutesDuration() {
        // 1h 23m 45s -> "1h 23m" (seconds dropped once we have hours)
        #expect(EpisodeFormatting.formatDuration(UInt32(3600 + 23 * 60 + 45)) == "1h 23m")
    }

    @Test func wholeHourShowsZeroMinutes() {
        #expect(EpisodeFormatting.formatDuration(2 * 3600) == "2h 0m")
    }

    // MARK: Pub date

    @Test func nilPubDateIsNil() {
        #expect(EpisodeFormatting.formatPubDate(nil) == nil)
    }

    /// Exact string is locale-dependent, so assert only that a value produces a
    /// non-empty rendering (the epoch, which is unambiguous across time zones).
    @Test func presentPubDateRenders() {
        let rendered = EpisodeFormatting.formatPubDate(0)
        #expect(rendered != nil)
        #expect(rendered?.isEmpty == false)
    }
}
