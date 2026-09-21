import App
import SwiftUI

/// Full-page view for a single episode. Renders the stored metadata, show notes,
/// and a working download control; transport controls, chapters, and bookmarks are
/// DEBUG-marked placeholders because their engines don't exist yet. Static metadata
/// is taken from the `episode` snapshot passed in from the list, but the download
/// state is read live from `core.view` so it updates while the page is open.
struct EpisodeDetailView: View {
    @ObservedObject var core: Core
    @Environment(\.themeColors) private var themeColors
    let episode: EpisodeSummary
    let feedTitle: String

    /// The current version of this episode from the core, so download-state changes
    /// show up here without re-navigating. Falls back to the injected snapshot if
    /// the list is no longer the selected feed.
    private var liveEpisode: EpisodeSummary {
        core.view.subscriptionDetail.episodes.first { $0.id == episode.id } ?? episode
    }

    /// The op-failure notice message, but only when it concerns this episode — the
    /// notice is subscription-wide, so scope it so one episode's failure doesn't
    /// surface on another's detail page.
    private var episodeNotice: String? {
        guard let notice = core.view.subscriptionDetail.downloadNotice,
              notice.episodeId == episode.id
        else {
            return nil
        }
        return notice.message
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                artworkHeader
                titleBlock
                playbackControls
                // A delete/cancel initiated here can fail; show the same non-blocking
                // notice the list uses so the failure is visible without navigating back
                // — but only when it's about this episode (see `episodeNotice`).
                if let notice = episodeNotice {
                    DownloadNoticeBanner(message: notice)
                }
                downloadSection
                Divider()
                showNotes
                PlaceholderSection(
                    title: "Chapters",
                    note: "Chapter parsing isn't built yet.",
                )
                PlaceholderSection(
                    title: "Bookmarks",
                    note: "Bookmarks need the playback engine.",
                )
            }
            .padding()
        }
        .background(themeColors.background)
        .navigationTitle(episode.title)
        .navigationBarTitleDisplayMode(.inline)
    }

    private var artworkHeader: some View {
        ArtworkView(urlString: episode.artworkUrl, size: 200)
            .frame(maxWidth: .infinity)
    }

    private var titleBlock: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(episode.title)
                .font(.title3)
                .fontWeight(.bold)
                .foregroundStyle(themeColors.text)
            Text(feedTitle)
                .font(.subheadline)
                .foregroundStyle(themeColors.secondaryText)
            if let meta = metaLine {
                Text(meta)
                    .font(.caption)
                    .foregroundStyle(themeColors.secondaryText)
            }
        }
    }

    /// Transport controls with no engine behind them — inert placeholders.
    private var playbackControls: some View {
        VStack(spacing: 10) {
            HStack(spacing: 24) {
                Image(systemName: "gobackward.15")
                    .font(.title2)
                    .stubbed()
                Image(systemName: "play.circle.fill")
                    .font(.system(size: 56))
                    .stubbed()
                Image(systemName: "goforward.30")
                    .font(.title2)
                    .stubbed()
            }
            ProgressView(value: 0)
                .tint(.debug)
                .disabled(true)
            HStack(spacing: 6) {
                Image(systemName: "nosign")
                    .font(.caption)
                    .foregroundStyle(Color.debug)
                Text("No playback engine yet")
                    .font(.caption2)
                    .foregroundStyle(themeColors.secondaryText)
            }
        }
        .frame(maxWidth: .infinity)
    }

    /// Working download control, driven by the live download status.
    @ViewBuilder private var downloadSection: some View {
        switch liveEpisode.downloadStatus {
        case .notDownloaded:
            Button { core.update(.downloadEpisode(episode.id)) } label: {
                Label("Download episode", systemImage: "arrow.down.circle")
            }
        case .queued:
            HStack {
                Label("Queued for download", systemImage: "clock")
                    .foregroundStyle(themeColors.secondaryText)
                Spacer()
                cancelButton
            }
        case .downloading:
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 6) {
                    if let progress = EpisodeFormatting.downloadProgress(
                        received: liveEpisode.downloadReceivedBytes,
                        total: liveEpisode.downloadTotalBytes,
                    ) {
                        if let fraction = progress.fraction {
                            ProgressView(value: fraction)
                        } else {
                            ProgressView()
                        }
                        Text(progress.label)
                            .font(.caption)
                            .monospacedDigit()
                            .foregroundStyle(themeColors.secondaryText)
                    } else {
                        Label {
                            Text("Downloading…")
                        } icon: {
                            ProgressView()
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                cancelButton
            }
        case .downloaded:
            HStack {
                Label("Downloaded", systemImage: "checkmark.circle.fill")
                    .foregroundStyle(themeColors.success)
                Spacer()
                Button(role: .destructive) {
                    core.update(.deleteDownload(episode.id))
                } label: {
                    Label("Delete", systemImage: "trash")
                }
            }
        case .failed:
            VStack(alignment: .leading, spacing: 6) {
                HStack {
                    Label("Download failed", systemImage: "exclamationmark.triangle")
                        .foregroundStyle(themeColors.error)
                    Spacer()
                    Button("Retry") { core.update(.downloadEpisode(episode.id)) }
                }
                if let reason = liveEpisode.downloadError {
                    Text(reason)
                        .font(.caption)
                        .foregroundStyle(themeColors.secondaryText)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
        case .removedFromFeed:
            Label("Removed from feed", systemImage: "xmark.circle")
                .foregroundStyle(themeColors.secondaryText)
        }
    }

    /// Small cancel affordance shown beside the progress bar (and the queued row).
    private var cancelButton: some View {
        Button(role: .destructive) {
            core.update(.cancelDownload(episode.id))
        } label: {
            Image(systemName: "xmark.circle.fill")
                .font(.title2)
        }
        .buttonStyle(.borderless)
        .accessibilityLabel("Cancel download")
    }

    private var showNotes: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Show notes")
                .font(.headline)
                .foregroundStyle(themeColors.text)
            if let description = episode.description, !description.isEmpty {
                ShowNotesText(html: description, fallback: episode.descriptionText)
            } else {
                Text("No show notes.")
                    .font(.body)
                    .foregroundStyle(themeColors.secondaryText)
            }
        }
    }

    private var metaLine: String? {
        EpisodeFormatting.metaLine(pubDate: episode.pubDate, durationSecs: episode.durationSecs)
    }
}

/// A feature section whose backend doesn't exist yet: DEBUG-tinted title + STUB
/// badge so it's clearly non-functional.
private struct PlaceholderSection: View {
    @Environment(\.themeColors) private var themeColors
    let title: String
    let note: String

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 8) {
                Text(title)
                    .font(.headline)
                    .foregroundStyle(Color.debug)
                Image(systemName: "nosign")
                    .foregroundStyle(Color.debug)
            }
            Text(note)
                .font(.caption)
                .foregroundStyle(themeColors.secondaryText)
        }
    }
}
