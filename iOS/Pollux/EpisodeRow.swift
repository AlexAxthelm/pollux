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

                metaRow

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

    /// The date·duration line — except while downloading, when it is replaced by a
    /// progress indicator. A downloaded episode gets a small filled-download glyph
    /// ahead of the date so status reads inline instead of on its own badge line.
    @ViewBuilder private var metaRow: some View {
        if episode.downloadStatus == .downloading {
            downloadingLine
        } else {
            HStack(spacing: 4) {
                if episode.downloadStatus == .downloaded {
                    Image(systemName: "arrow.down.circle.fill")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .accessibilityLabel("Downloaded")
                }
                if let meta = metaLine {
                    Text(meta)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
    }

    /// In-progress download: a determinate bar when the size is known, otherwise a
    /// spinner with whatever byte count we have. Replaces the date·duration line.
    @ViewBuilder private var downloadingLine: some View {
        let progress = EpisodeFormatting.downloadProgress(
            received: episode.downloadReceivedBytes,
            total: episode.downloadTotalBytes,
        )
        if let progress, let fraction = progress.fraction {
            ProgressView(value: fraction)
                .accessibilityLabel("Downloading \(Int(fraction * 100)) percent")
        } else {
            HStack(spacing: 6) {
                ProgressView()
                Text(progress?.label ?? "Downloading…")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .accessibilityLabel("Downloading \(progress?.label ?? "")")
        }
    }

    @ViewBuilder private var statusRow: some View {
        let playback = playbackBadge
        if playback != nil || playbackPositionText != nil {
            HStack(spacing: 10) {
                if let playback {
                    StatusBadge(systemImage: playback.icon, text: playback.text)
                }
                if let positionText = playbackPositionText {
                    StatusBadge(systemImage: "clock.arrow.circlepath", text: positionText)
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
            // Progress lives on the meta line while downloading, so the trailing
            // slot stays empty rather than duplicating it with a percentage.
            EmptyView()
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
