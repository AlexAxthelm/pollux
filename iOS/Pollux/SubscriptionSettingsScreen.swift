import App
import SwiftUI

/// One subscription's settings page, reached from the gear on its page. It is marked as
/// editing that podcast (its artwork and name head the page) so it can't be mistaken for the
/// library-wide settings. A value the podcast hasn't set follows the library setting and
/// reads "(inherited from library)"; one it has set shows a button that returns it to the library value.
///
/// The rows read from `core.view.subscriptionDetail.settings`, which describes the selected
/// subscription, so the page shows them only while that is this subscription (it is, when
/// pushed from the subscription page). If the selection is ever another subscription's, the page
/// selects its own and shows a spinner meanwhile, with a note to go back and reopen if it doesn't
/// clear, rather than an empty page.
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
                    unsetLabel: "(inherited from library)",
                    onChange: {
                        core.update(.setSubscriptionSetting(subscriptionId: subscription.id, change: $0))
                    },
                )
            } else {
                loading
            }
        }
        // If the core has another subscription selected (it should not, when pushed from the
        // subscription page), select this one, as the subscription page does when it appears.
        .task(id: isSelected) {
            if !isSelected {
                core.update(.selectSubscription(subscription.id))
            }
        }
        .scrollContentBackground(.hidden)
        .background(themeColors.background)
        .navigationTitle("Podcast settings")
        .navigationBarTitleDisplayMode(.inline)
    }

    /// Shown instead of the settings while the core's selection is a different subscription,
    /// so the page never sits empty and unexplained (and never shows another podcast's values).
    /// The page selects its own subscription, which normally ends this at once; the message is
    /// for when it doesn't, since going back and opening the settings again also works.
    private var loading: some View {
        Section {
            VStack(spacing: 8) {
                ProgressView()
                    .accessibilityLabel("Loading podcast settings")
                Text("If this isn't loading, try going back and reopening the settings.")
                    .font(.caption)
                    .foregroundStyle(themeColors.secondaryText)
                    .multilineTextAlignment(.center)
            }
            .frame(maxWidth: .infinity)
        }
        .listRowBackground(themeColors.secondaryBackground)
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
