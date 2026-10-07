import Foundation

/// Display strings for the player's clock and skip buttons. Kept out of the views so
/// it can be unit-tested without SwiftUI (like `EpisodeFormatting`).
enum PlayerFormatting {
    /// "2:03" under an hour, "1:02:03" from an hour up.
    static func clock(_ seconds: UInt32) -> String {
        let total = Int(seconds)
        let hours = total / 3600
        let minutes = (total % 3600) / 60
        let secs = total % 60
        if hours > 0 {
            return String(format: "%d:%02d:%02d", hours, minutes, secs)
        }
        return String(format: "%d:%02d", minutes, secs)
    }

    /// Time left as "-2:03", never negative (a position at or past the end reads "-0:00").
    static func remaining(position: UInt32, duration: UInt32) -> String {
        "-" + clock(duration > position ? duration - position : 0)
    }

    /// The skip button's label: "+30" forward, "-15" back.
    static func skipLabel(_ seconds: UInt32, forward: Bool) -> String {
        (forward ? "+" : "-") + String(seconds)
    }

    /// Spoken form for VoiceOver: "1 hour 2 minutes 3 seconds".
    static func spoken(_ seconds: UInt32) -> String {
        spokenFormatter.string(from: TimeInterval(seconds)) ?? "\(seconds) seconds"
    }

    /// Built once: creating a formatter is costly, and the scrubber's accessibility value
    /// asks for this on every update (every tick, and every step of a drag).
    private static let spokenFormatter: DateComponentsFormatter = {
        let formatter = DateComponentsFormatter()
        formatter.unitsStyle = .full
        formatter.allowedUnits = [.hour, .minute, .second]
        return formatter
    }()

    /// Fraction of the episode played, 0...1; 0 when the duration is unknown.
    static func fraction(position: UInt32, duration: UInt32?) -> Double {
        guard let duration, duration > 0 else { return 0 }
        return min(1, Double(position) / Double(duration))
    }
}
