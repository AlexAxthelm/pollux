import App
import SwiftUI

/// Full-page view for a single episode. Renders the stored metadata, show notes,
/// and a working download control; transport controls, chapters, and bookmarks are
/// DEBUG-marked placeholders because their engines don't exist yet. Static metadata
/// is taken from the `episode` snapshot passed in from the list, but the download
/// state is read live from `core.view` so it updates while the page is open.
struct EpisodeDetailView: View {
    @ObservedObject var core: Core
    let episode: EpisodeSummary
    let feedTitle: String

    /// The current version of this episode from the core, so download-state changes
    /// show up here without re-navigating. Falls back to the injected snapshot if
    /// the list is no longer the selected feed.
    private var liveEpisode: EpisodeSummary {
        core.view.subscriptionDetail.episodes.first { $0.id == episode.id } ?? episode
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                artworkHeader
                titleBlock
                playbackControls
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
            Text(feedTitle)
                .font(.subheadline)
                .foregroundStyle(.secondary)
            if let meta = metaLine {
                Text(meta)
                    .font(.caption)
                    .foregroundStyle(.secondary)
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
                    .foregroundStyle(.secondary)
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
            Label("Queued for download", systemImage: "clock")
                .foregroundStyle(.secondary)
        case .downloading:
            Label {
                Text("Downloading…")
            } icon: {
                ProgressView()
            }
        case .downloaded:
            HStack {
                Label("Downloaded", systemImage: "checkmark.circle.fill")
                    .foregroundStyle(.green)
                Spacer()
                Button(role: .destructive) {
                    core.update(.deleteDownload(episode.id))
                } label: {
                    Label("Delete", systemImage: "trash")
                }
            }
        case .failed:
            HStack {
                Label("Download failed", systemImage: "exclamationmark.triangle")
                    .foregroundStyle(.orange)
                Spacer()
                Button("Retry") { core.update(.downloadEpisode(episode.id)) }
            }
        case .removedFromFeed:
            Label("Removed from feed", systemImage: "xmark.circle")
                .foregroundStyle(.secondary)
        }
    }

    private var showNotes: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Show notes")
                .font(.headline)
            if let description = episode.description, !description.isEmpty {
                ShowNotesText(html: description, fallback: episode.descriptionText)
            } else {
                Text("No show notes.")
                    .font(.body)
                    .foregroundStyle(.secondary)
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
                .foregroundStyle(.secondary)
        }
    }
}
