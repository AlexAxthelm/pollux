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

    // MARK: Download progress

    @Test func nilReceivedHasNoProgress() {
        #expect(EpisodeFormatting.downloadProgress(received: nil, total: 100) == nil)
    }

    @Test func knownTotalGivesFractionAndCombinedLabel() {
        let progress = EpisodeFormatting.downloadProgress(received: 50, total: 100)
        #expect(progress?.fraction == 0.5)
        #expect(progress?.label == "\(bytes(50)) / \(bytes(100))")
    }

    @Test func unknownTotalHasNoFractionAndReceivedOnlyLabel() {
        let progress = EpisodeFormatting.downloadProgress(received: 1234, total: nil)
        #expect(progress?.fraction == nil)
        #expect(progress?.label == bytes(1234))
    }

    @Test func zeroTotalIsTreatedAsUnknown() {
        // A reported total of 0 carries no information, so it behaves like `nil`.
        let progress = EpisodeFormatting.downloadProgress(received: 1234, total: 0)
        #expect(progress?.fraction == nil)
        #expect(progress?.label == bytes(1234))
    }

    @Test func receivedExceedingTotalClampsFractionToOne() {
        // A late byte count past the advertised size shouldn't push the bar past full.
        let progress = EpisodeFormatting.downloadProgress(received: 150, total: 100)
        #expect(progress?.fraction == 1.0)
        #expect(progress?.label == "\(bytes(150)) / \(bytes(100))")
    }

    @Test func hugeByteCountsDoNotOverflow() {
        // Int64(clamping:) keeps the ByteCountFormatter input in range, and the
        // fraction still resolves to a sane clamped value.
        let progress = EpisodeFormatting.downloadProgress(received: .max, total: .max)
        #expect(progress?.fraction == 1.0)
        #expect(progress?.label == "\(bytes(.max)) / \(bytes(.max))")
    }
}

/// Mirror of `EpisodeFormatting`'s private byte formatter so tests can assert exact
/// labels without depending on the current locale's spelling of a unit.
private func bytes(_ count: Int64) -> String {
    let formatter = ByteCountFormatter()
    formatter.countStyle = .file
    return formatter.string(fromByteCount: count)
}
