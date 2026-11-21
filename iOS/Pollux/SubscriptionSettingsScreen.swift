import App
import SwiftUI

/// One subscription's settings page, reached from the gear on its page. It is marked as
/// editing that podcast (its artwork and name head the page) so it can't be mistaken for the
/// library-wide settings. A value the podcast hasn't set follows the library setting and
/// reads "(default)"; one it has set shows a button that returns it to the library value.
///
/// The rows read from `core.view.subscriptionDetail.settings`, which describes the selected
/// subscription, so the page shows them only while that is this subscription (it is, when
/// pushed from the subscription page).
struct SubscriptionSettingsScreen: View {
    @ObservedObject var core: Core
    @Environment(\.themeColors) private var themeColors
    let subscription: SubscriptionSummary

    private var isSelected: Bool {
        core.view.subscriptionDetail.subscriptionId == subscription.id
    }

    var body: some View {
        Form {
            header
            if isSelected {
                PlaybackSettingsSection(
                    settings: core.view.subscriptionDetail.settings,
                    footer: "Anything you don't change here follows the library settings.",
                    onChange: {
                        core.update(.setSubscriptionSetting(subscriptionId: subscription.id, change: $0))
                    },
                )
            }
        }
        .scrollContentBackground(.hidden)
        .background(themeColors.background)
        .navigationTitle("Podcast settings")
        .navigationBarTitleDisplayMode(.inline)
    }

    private var header: some View {
        Section {
            HStack(spacing: 12) {
                ArtworkView(urlString: subscription.artworkUrl, size: 48)
                VStack(alignment: .leading, spacing: 2) {
                    Text(subscription.title)
                        .font(.headline)
                        .foregroundStyle(themeColors.text)
                        .lineLimit(2)
                    Text("Changes here apply to this podcast only.")
                        .font(.caption)
                        .foregroundStyle(themeColors.secondaryText)
                }
            }
            .accessibilityElement(children: .combine)
        }
        .listRowBackground(themeColors.secondaryBackground)
    }
}

#Preview {
    NavigationStack {
        SubscriptionSettingsScreen(
            core: Core(),
            subscription: SubscriptionSummary(
                id: "preview",
                title: "A Podcast",
                artworkUrl: nil,
                refreshing: false,
                refreshError: nil,
            ),
        )
    }
}
