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

    /// Live download progress for display. Returns nil before any bytes are known.
    /// `fraction` is 0...1 when the total size is known (nil → show an indeterminate
    /// indicator); `label` is a byte summary like "12.3 MB / 45.6 MB", or just
    /// "12.3 MB" when the server didn't report a total.
    static func downloadProgress(
        received: UInt64?, total: UInt64?,
    ) -> (fraction: Double?, label: String)? {
        guard let received else { return nil }
        let receivedText = byteFormatter.string(fromByteCount: Int64(clamping: received))
        if let total, total > 0 {
            let totalText = byteFormatter.string(fromByteCount: Int64(clamping: total))
            let fraction = min(1.0, Double(received) / Double(total))
            return (fraction, "\(receivedText) / \(totalText)")
        }
        return (nil, receivedText)
    }

    private static let pubDateFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .none
        return formatter
    }()

    private static let byteFormatter: ByteCountFormatter = {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter
    }()
}
