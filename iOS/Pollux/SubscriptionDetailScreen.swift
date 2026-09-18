import App
import SwiftUI

/// The subscription details page: the episode list for one feed. Data flows the
/// same way as the subscribe flow — selecting a subscription dispatches an Event,
/// the core runs `ListEpisodesBySubscription`, and the result comes back through
/// `core.view.subscriptionDetail`. The sort menu is a real feature (pure core
/// logic); the playback/download controls in each row are placeholders.
struct SubscriptionDetailScreen: View {
    @ObservedObject var core: Core
    let subscription: SubscriptionSummary

    private var detail: SubscriptionDetailView {
        core.view.subscriptionDetail
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            content
        }
        .navigationTitle(subscription.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                sortMenu
            }
        }
        .navigationDestination(for: EpisodeSummary.self) { episode in
            EpisodeDetailView(core: core, episode: episode, feedTitle: subscription.title)
        }
        .task(id: subscription.id) {
            core.update(.selectSubscription(subscription.id))
        }
    }

    private var header: some View {
        HStack(spacing: 12) {
            ArtworkView(urlString: subscription.artworkUrl, size: 72)
            VStack(alignment: .leading, spacing: 4) {
                Text(subscription.title)
                    .font(.headline)
                    .lineLimit(2)
                Text(episodeCountText)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
        }
        .padding()
    }

    @ViewBuilder private var content: some View {
        if detail.loading {
            centered { ProgressView() }
        } else if let error = detail.error {
            centered {
                Text(error)
                    .foregroundStyle(.red)
                    .font(.callout)
                    .multilineTextAlignment(.center)
                    .padding()
            }
        } else if detail.episodes.isEmpty {
            centered {
                Text("No episodes")
                    .foregroundStyle(.secondary)
            }
        } else {
            List(detail.episodes, id: \.id) { episode in
                NavigationLink(value: episode) {
                    EpisodeRow(
                        episode: episode,
                        onDownload: { core.update(.downloadEpisode($0)) },
                        onDeleteDownload: { core.update(.deleteDownload($0)) },
                        onCancelDownload: { core.update(.cancelDownload($0)) },
                    )
                }
                // Spec default: short swipe-right → Download / Delete. This is the
                // one actionable swipe today (flag / mark-played have no engine yet);
                // it becomes user-configurable when Settings lands.
                .swipeActions(edge: .leading, allowsFullSwipe: fullSwipeAllowed(episode)) {
                    downloadSwipeButton(for: episode)
                }
            }
            .listStyle(.plain)
        }
    }

    /// Leading-swipe download action, mirroring the row's menu: an action for the
    /// states where one applies, nothing for the rest (there is no cancel yet).
    @ViewBuilder private func downloadSwipeButton(for episode: EpisodeSummary) -> some View {
        switch episode.downloadStatus {
        case .notDownloaded:
            Button { core.update(.downloadEpisode(episode.id)) } label: {
                Label("Download", systemImage: "arrow.down.circle")
            }
            .tint(.blue)
        case .failed:
            Button { core.update(.downloadEpisode(episode.id)) } label: {
                Label("Retry", systemImage: "arrow.clockwise.circle")
            }
            .tint(.orange)
        case .downloaded:
            Button(role: .destructive) { core.update(.deleteDownload(episode.id)) } label: {
                Label("Delete", systemImage: "trash")
            }
        case .queued, .downloading:
            Button(role: .destructive) { core.update(.cancelDownload(episode.id)) } label: {
                Label("Cancel", systemImage: "xmark.circle")
            }
        case .removedFromFeed:
            EmptyView()
        }
    }

    /// Full-swipe commits the action without tapping. Allowed for downloading (safe,
    /// re-downloadable) but not for deleting, so a stray long swipe can't wipe a file.
    private func fullSwipeAllowed(_ episode: EpisodeSummary) -> Bool {
        switch episode.downloadStatus {
        case .notDownloaded, .failed: true
        default: false
        }
    }

    private var sortMenu: some View {
        Menu {
            Picker("Sort", selection: sortBinding) {
                Text("Newest first").tag(EpisodeSortOrder.pubDateDesc)
                Text("Oldest first").tag(EpisodeSortOrder.pubDateAsc)
                Text("Title A–Z").tag(EpisodeSortOrder.titleAsc)
            }
        } label: {
            Label("Sort", systemImage: "arrow.up.arrow.down")
        }
        .disabled(detail.episodes.isEmpty)
    }

    private var sortBinding: Binding<EpisodeSortOrder> {
        Binding(
            get: { detail.sortOrder },
            set: { core.update(.setEpisodeSort($0)) },
        )
    }

    private var episodeCountText: String {
        let count = detail.episodes.count
        return count == 1 ? "1 episode" : "\(count) episodes"
    }

    private func centered(@ViewBuilder _ body: () -> some View) -> some View {
        body()
            .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
