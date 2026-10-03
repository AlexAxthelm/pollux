import SwiftUI

/// A non-blocking warning banner. Unlike a load error, which replaces the screen's
/// content, it leaves the surrounding content and controls in place; the core decides
/// when each notice clears. Used for:
///
/// - a failed download *operation* (a persistence write that didn't commit, or a file
///   that couldn't be removed), shown on the subscription detail list and on a single
///   episode's detail page; cleared by the next download action or a feed switch;
/// - the detail list's "couldn't update the episode list" notice, when a reload after a
///   refresh failed and the rows on screen may be out of date; cleared by a successful
///   reload or a feed switch.
struct NoticeBanner: View {
    @Environment(\.themeColors) private var themeColors
    let message: String

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            // Decorative: the message text conveys the warning, so keep VoiceOver from
            // announcing the icon's name ahead of it.
            Image(systemName: "exclamationmark.triangle.fill")
                .accessibilityHidden(true)
            Text(message)
                .font(.caption)
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 0)
        }
        .foregroundStyle(themeColors.error)
        .padding(.horizontal)
        .padding(.vertical, 8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(themeColors.secondaryBackground)
    }
}
