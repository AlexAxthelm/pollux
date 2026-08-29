import App
import SwiftUI

/// Full-page view for a single episode. Renders the stored metadata and show
/// notes; the transport controls, chapters, and bookmarks are DEBUG-marked
/// placeholders because their engines (playback, chapters, bookmarks) don't exist
/// yet. The episode is passed in from the list rather than re-fetched — the list's
/// ViewModel already carries it. A dedicated `GetEpisode` round-trip could replace
/// this later if the detail page ever needs data the list doesn't ship.
struct EpisodeDetailView: View {
    let episode: EpisodeSummary
    let feedTitle: String

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                artworkHeader
                titleBlock
                playbackControls
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

    private var showNotes: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Show notes")
                .font(.headline)
            if let description = episode.description, !description.isEmpty {
                Text(description)
                    .font(.body)
                    .foregroundStyle(.primary)
            } else {
                Text("No show notes.")
                    .font(.body)
                    .foregroundStyle(.secondary)
            }
        }
    }

    /// "Aug 29, 2026 · 45m" — whichever parts are present.
    private var metaLine: String? {
        let parts = [
            EpisodeFormatting.formatPubDate(episode.pubDate),
            EpisodeFormatting.formatDuration(episode.durationSecs),
        ].compactMap(\.self)
        return parts.isEmpty ? nil : parts.joined(separator: " · ")
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
