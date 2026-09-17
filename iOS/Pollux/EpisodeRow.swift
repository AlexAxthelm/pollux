import App
import SwiftUI

/// One episode in the subscription details list. Shows stored data (art, title,
/// date, duration, description, read-only played/download status) and a working
/// download control. Playback and more-actions remain DEBUG-tinted, disabled
/// placeholders because their engines don't exist yet — see `DebugStyle.swift`.
struct EpisodeRow: View {
    let episode: EpisodeSummary
    /// Queue this episode for download (or retry a failed one).
    let onDownload: (String) -> Void
    /// Remove this episode's downloaded file.
    let onDeleteDownload: (String) -> Void

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
                        .foregroundStyle(.secondary)
                }

                if let description = episode.descriptionText, !description.isEmpty {
                    Text(description)
                        .font(.caption)
                        .foregroundStyle(.secondary)
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

    /// Trailing controls: a working download control plus the still-stubbed play
    /// and more-actions buttons (their engines don't exist yet). `.borderless`
    /// keeps the download button from also triggering the row's NavigationLink.
    private var placeholderControls: some View {
        HStack(spacing: 12) {
            Image(systemName: "play.circle.fill")
                .font(.title2)
                .stubbed()
            downloadControl
            Image(systemName: "ellipsis.circle")
                .font(.title2)
                .stubbed()
        }
    }

    /// The download affordance, driven by the episode's current download status.
    /// Actionable states are real buttons; transient/terminal states are indicators.
    @ViewBuilder private var downloadControl: some View {
        switch episode.downloadStatus {
        case .notDownloaded:
            Button { onDownload(episode.id) } label: {
                Image(systemName: "arrow.down.circle").font(.title2)
            }
            .buttonStyle(.borderless)
            .accessibilityLabel("Download episode")
        case .failed:
            Button { onDownload(episode.id) } label: {
                Image(systemName: "arrow.clockwise.circle").font(.title2)
            }
            .buttonStyle(.borderless)
            .tint(.orange)
            .accessibilityLabel("Retry download")
        case .queued:
            Image(systemName: "clock")
                .font(.title2)
                .foregroundStyle(.secondary)
                .accessibilityLabel("Queued for download")
        case .downloading:
            if let progress = EpisodeFormatting.downloadProgress(
                received: episode.downloadReceivedBytes,
                total: episode.downloadTotalBytes,
            ), let fraction = progress.fraction {
                Text("\(Int(fraction * 100))%")
                    .font(.caption2)
                    .monospacedDigit()
                    .foregroundStyle(.secondary)
                    .accessibilityLabel("Downloading \(Int(fraction * 100)) percent")
            } else {
                ProgressView()
                    .accessibilityLabel("Downloading")
            }
        case .downloaded:
            Button { onDeleteDownload(episode.id) } label: {
                Image(systemName: "trash.circle").font(.title2)
            }
            .buttonStyle(.borderless)
            .tint(.gray)
            .accessibilityLabel("Delete download")
        case .removedFromFeed:
            Image(systemName: "xmark.circle")
                .font(.title2)
                .foregroundStyle(.secondary)
                .accessibilityLabel("Removed from feed")
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

/// A compact read-only status indicator (real data, semantic colors).
private struct StatusBadge: View {
    let systemImage: String
    let text: String

    var body: some View {
        Label(text, systemImage: systemImage)
            .font(.caption2)
            .foregroundStyle(.secondary)
            .labelStyle(.titleAndIcon)
    }
}
