import App
import SwiftUI

/// A library row: the feed title, plus a spinner while it is queued for or being
/// refreshed and a warning marker when its last refresh failed. The failure reason is
/// exposed to VoiceOver rather than shown inline, keeping the list compact.
struct SubscriptionRow: View {
    @Environment(\.themeColors) private var themeColors
    let subscription: SubscriptionSummary

    var body: some View {
        HStack(spacing: 8) {
            Text(subscription.title)
                .foregroundStyle(themeColors.text)
            Spacer(minLength: 0)
            if subscription.refreshing {
                ProgressView()
                    .controlSize(.small)
            } else if subscription.refreshError != nil {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(themeColors.warning)
                    .accessibilityHidden(true)
            }
        }
        .accessibilityElement(children: .combine)
        .accessibilityValue(accessibilityValue)
    }

    private var accessibilityValue: String {
        if subscription.refreshing {
            "Refreshing"
        } else if let error = subscription.refreshError {
            "Last refresh failed: \(error)"
        } else {
            ""
        }
    }
}
