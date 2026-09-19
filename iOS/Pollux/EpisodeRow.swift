import App
import SwiftUI

/// One episode in the subscription details list. Shows stored data (art, title,
/// date, duration, description, read-only played/download status) and a working
/// download control. Playback and more-actions remain DEBUG-tinted, disabled
/// placeholders because their engines don't exist yet — see `DebugStyle.swift`.
struct EpisodeRow: View {
    @Environment(\.themeColors) private var themeColors
    let episode: EpisodeSummary
    /// Queue this episode for download (or retry a failed one).
    let onDownload: (String) -> Void
    /// Remove this episode's downloaded file.
    let onDeleteDownload: (String) -> Void
    /// Stop this episode's in-flight or queued download.
    let onCancelDownload: (String) -> Void

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            ArtworkView(urlString: episode.artworkUrl, size: 56)

            VStack(alignment: .leading, spacing: 4) {
                Text(episode.title)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .foregroundStyle(themeColors.text)
                    .lineLimit(2)

                metaRow

                if let description = episode.descriptionText, !description.isEmpty {
                    Text(description)
                        .font(.caption)
                        .foregroundStyle(themeColors.secondaryText)
                        .lineLimit(1)
                }

                statusRow
            }

            Spacer(minLength: 8)

            trailingControls
        }
        .padding(.vertical, 4)
    }

    private var metaLine: String? {
        EpisodeFormatting.metaLine(pubDate: episode.pubDate, durationSecs: episode.durationSecs)
    }

    /// The date·duration line — except while downloading, when it is replaced by a
    /// progress indicator. Non-not-downloaded states get a small glyph ahead of the
    /// date so status reads inline instead of on its own badge line — otherwise
    /// queued/failed/removed rows would look identical to a plain undownloaded one.
    @ViewBuilder private var metaRow: some View {
        if episode.downloadStatus == .downloading {
            downloadingLine
        } else {
            HStack(spacing: 4) {
                if let glyph = statusGlyph {
                    Image(systemName: glyph.icon)
                        .font(.caption)
                        .foregroundStyle(glyph.tint)
                        .accessibilityLabel(glyph.label)
                }
                if let meta = metaLine {
                    Text(meta)
                        .font(.caption)
                        .foregroundStyle(themeColors.secondaryText)
                }
            }
        }
    }

    /// The inline download-status glyph for the date line. `nil` for not-downloaded
    /// (a plain date) and downloading (handled by `downloadingLine`). Failed uses the
    /// error color to stand out; the rest are neutral.
    private var statusGlyph: StatusGlyph? {
        switch episode.downloadStatus {
        case .downloaded:
            StatusGlyph(icon: "arrow.down.circle.fill", tint: themeColors.secondaryText, label: "Downloaded")
        case .queued:
            StatusGlyph(icon: "clock", tint: themeColors.secondaryText, label: "Queued for download")
        case .failed:
            StatusGlyph(icon: "exclamationmark.triangle.fill", tint: themeColors.error, label: "Download failed")
        case .removedFromFeed:
            StatusGlyph(icon: "xmark.circle", tint: themeColors.secondaryText, label: "Removed from feed")
        case .notDownloaded, .downloading:
            nil
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
                    .foregroundStyle(themeColors.secondaryText)
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

    /// Trailing controls: the still-stubbed play button (no playback engine yet) and
    /// the three-dots menu, which is where the download action now lives. Download
    /// *status* stays inline (`metaRow`); this menu is the *action* surface.
    private var trailingControls: some View {
        HStack(spacing: 12) {
            Image(systemName: "play.circle.fill")
                .font(.title2)
                .stubbed()
            moreMenu
        }
    }

    /// The three-dots menu. Today it holds only the download action (its first,
    /// context-sensitive item); Play, Mark played, Flag, etc. join it as their
    /// engines land. Swipe shortcuts are a later, user-configurable addition.
    private var moreMenu: some View {
        Menu {
            downloadMenuItem
        } label: {
            Image(systemName: "ellipsis.circle")
                .font(.title2)
        }
        .accessibilityLabel("More actions")
    }

    /// The download entry, driven by status: an action for the states where one
    /// applies, or a disabled indicator otherwise (there is no cancel yet).
    @ViewBuilder private var downloadMenuItem: some View {
        switch episode.downloadStatus {
        case .notDownloaded:
            Button { onDownload(episode.id) } label: {
                Label("Download", systemImage: "arrow.down.circle")
            }
        case .failed:
            Button { onDownload(episode.id) } label: {
                Label("Retry Download", systemImage: "arrow.clockwise.circle")
            }
        case .downloaded:
            Button(role: .destructive) { onDeleteDownload(episode.id) } label: {
                Label("Delete Download", systemImage: "trash")
            }
        case .queued, .downloading:
            Button(role: .destructive) { onCancelDownload(episode.id) } label: {
                Label("Cancel Download", systemImage: "xmark.circle")
            }
        case .removedFromFeed:
            Button {} label: { Label("Removed From Feed", systemImage: "xmark.circle") }
                .disabled(true)
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

/// Inline download-status glyph shown before the date on an episode row.
private struct StatusGlyph {
    let icon: String
    let tint: Color
    let label: String
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
