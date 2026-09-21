import SwiftUI

/// Non-blocking banner for a failed download *operation* (a persistence write that
/// didn't commit, or a file that couldn't be removed). Unlike a list-load error it
/// leaves the surrounding content and controls in place; the core clears the notice
/// on the next download action or feed switch. Shown on both the subscription detail
/// list and a single episode's detail page.
struct DownloadNoticeBanner: View {
    @Environment(\.themeColors) private var themeColors
    let message: String

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Image(systemName: "exclamationmark.triangle.fill")
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
