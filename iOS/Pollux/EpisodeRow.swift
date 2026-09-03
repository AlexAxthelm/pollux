import App
import SwiftUI

/// One episode in the subscription details list. Shows only stored data (art,
/// title, date, duration, description, read-only played/download status). Controls
/// whose engines don't exist yet (play, download, more-actions) are rendered as
/// DEBUG-tinted, disabled placeholders — see `DebugStyle.swift`.
struct EpisodeRow: View {
    @Environment(\.themeColors) private var themeColors
    let episode: EpisodeSummary

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            ArtworkView(urlString: episode.artworkUrl, size: 56)

            VStack(alignment: .leading, spacing: 4) {
                Text(episode.title)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .lineLimit(2)

                if let meta = metaLine {
                    Text(meta)
                        .font(.caption)
                        .foregroundStyle(themeColors.secondaryText)
                }

                if let description = episode.descriptionText, !description.isEmpty {
                    Text(description)
                        .font(.caption)
                        .foregroundStyle(themeColors.secondaryText)
                        .lineLimit(1)
                }

                statusRow
            }

            Spacer(minLength: 8)

            placeholderControls
        }
        .padding(.vertical, 4)
    }

    private var metaLine: String? {
        EpisodeFormatting.metaLine(pubDate: episode.pubDate, durationSecs: episode.durationSecs)
    }

    @ViewBuilder private var statusRow: some View {
        let playback = playbackBadge
        let download = downloadBadge
        if playback != nil || download != nil || playbackPositionText != nil {
            HStack(spacing: 10) {
                if let playback {
                    StatusBadge(systemImage: playback.icon, text: playback.text)
                }
                if let positionText = playbackPositionText {
                    StatusBadge(systemImage: "clock.arrow.circlepath", text: positionText)
                }
                if let download {
                    StatusBadge(systemImage: download.icon, text: download.text)
                }
            }
            .padding(.top, 2)
        }
    }

    /// Play + more-actions: no playback engine or download manager exists yet, so
    /// these are inert placeholders (DEBUG tint + 🚫 overlay), not wired to fake
    /// behavior.
    private var placeholderControls: some View {
        HStack(spacing: 12) {
            Image(systemName: "play.circle.fill")
                .font(.title2)
                .stubbed()
            Image(systemName: "ellipsis.circle")
                .font(.title2)
                .stubbed()
        }
    }

    private var playbackBadge: (icon: String, text: String)? {
        switch episode.playbackStatus {
        case .played:
            ("checkmark.circle.fill", "Played")
        case .inProgress:
            ("pause.circle", "In progress")
        case .unplayed:
            nil
        }
    }

    /// Only meaningful for in-progress episodes; nothing sets a position yet, so
    /// in practice this stays nil until the playback engine lands.
    private var playbackPositionText: String? {
        guard episode.playbackStatus == .inProgress,
              let position = EpisodeFormatting.formatDuration(episode.playbackPositionSecs)
        else {
            return nil
        }
        return "at \(position)"
    }

    private var downloadBadge: (icon: String, text: String)? {
        switch episode.downloadStatus {
        case .notDownloaded:
            nil
        case .queued:
            ("clock", "Queued")
        case .downloading:
            ("arrow.down.circle", "Downloading")
        case .downloaded:
            ("arrow.down.circle.fill", "Downloaded")
        case .failed:
            ("exclamationmark.triangle", "Failed")
        case .removedFromFeed:
            ("xmark.circle", "Removed")
        }
    }
}

/// A compact read-only status indicator (real data, theme colors).
private struct StatusBadge: View {
    @Environment(\.themeColors) private var themeColors
    let systemImage: String
    let text: String

    var body: some View {
        Label(text, systemImage: systemImage)
            .font(.caption2)
            .foregroundStyle(themeColors.secondaryText)
            .labelStyle(.titleAndIcon)
    }
}
