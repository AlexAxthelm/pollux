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
            EpisodeDetailView(episode: episode, feedTitle: subscription.title)
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
                    EpisodeRow(episode: episode)
                }
            }
            .listStyle(.plain)
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
