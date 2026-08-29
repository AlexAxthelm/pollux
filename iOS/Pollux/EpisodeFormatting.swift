import Foundation

/// Locale-aware rendering of the raw episode values the core ships (`pub_date` as
/// a unix timestamp, `duration_secs` as seconds). The core owns *which* data and
/// its ordering; formatting for display is a shell concern, kept here so it can be
/// unit-tested without SwiftUI (mirroring `SubscribeFlow`).
enum EpisodeFormatting {
    /// A unix timestamp (seconds) as a medium-style local date, or nil when absent.
    static func formatPubDate(_ timestamp: Int64?) -> String? {
        guard let timestamp else { return nil }
        let date = Date(timeIntervalSince1970: TimeInterval(timestamp))
        return pubDateFormatter.string(from: date)
    }

    /// "Aug 29, 2026 · 45m" — the formatted pub date and duration joined by a
    /// middot, using whichever parts are present. Nil when neither is.
    static func metaLine(pubDate: Int64?, durationSecs: UInt32?) -> String? {
        let parts = [formatPubDate(pubDate), formatDuration(durationSecs)].compactMap(\.self)
        return parts.isEmpty ? nil : parts.joined(separator: " · ")
    }

    /// A compact duration like "1h 23m", "45m", or "30s". Nil when absent or zero.
    static func formatDuration(_ seconds: UInt32?) -> String? {
        guard let seconds, seconds > 0 else { return nil }
        let total = Int(seconds)
        let hours = total / 3600
        let minutes = (total % 3600) / 60
        let secs = total % 60
        if hours > 0 {
            return "\(hours)h \(minutes)m"
        }
        if minutes > 0 {
            return "\(minutes)m"
        }
        return "\(secs)s"
    }

    private static let pubDateFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .none
        return formatter
    }()
}
