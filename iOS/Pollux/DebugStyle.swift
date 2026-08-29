import SwiftUI

/// A deliberately loud, non-semantic color reserved for UI that is wired to
/// nothing yet (play, download, chapters, bookmarks — engines that don't exist).
///
/// This is a stopgap until the base16 theming system lands (see docs/features/
/// theme.md); real UI must use semantic roles (`.primary`/`.secondary`/`.tint`),
/// never this. Anything tinted `.debug` is a promise that it does nothing.
extension Color {
    static let debug = Color(red: 0.85, green: 0.1, blue: 0.55)
}

private struct StubModifier: ViewModifier {
    func body(content: Content) -> some View {
        content
            .foregroundStyle(Color.debug)
            .overlay {
                // Inherits the control's font, so the "no" sign scales to match
                // whatever it's marking. Semi-transparent so the control shows
                // through.
                Image(systemName: "nosign")
                    .foregroundStyle(.primary)
                    .opacity(0.55)
            }
            .disabled(true)
            .accessibilityLabel("Not functional yet")
    }
}

extension View {
    /// Marks a control as a non-functional placeholder: tints it the DEBUG color,
    /// overlays a semi-transparent 🚫, and disables interaction. The single place
    /// "this does nothing yet" is expressed, so stubs stay consistent and are
    /// trivially removed once the real engine lands.
    func stubbed() -> some View {
        modifier(StubModifier())
    }
}
