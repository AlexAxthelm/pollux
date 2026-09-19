import SwiftUI

/// Square podcast/episode artwork loaded from a URL, with a neutral placeholder
/// while loading or when the URL is missing or fails.
struct ArtworkView: View {
    @Environment(\.themeColors) private var themeColors
    let urlString: String?
    var size: CGFloat = 56

    var body: some View {
        artwork
            .frame(width: size, height: size)
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .strokeBorder(themeColors.secondaryBackground, lineWidth: 0.5),
            )
    }

    @ViewBuilder private var artwork: some View {
        if let urlString, let url = URL(string: urlString) {
            AsyncImage(url: url) { phase in
                switch phase {
                case let .success(image):
                    image.resizable().scaledToFill()
                case .failure:
                    placeholder
                default:
                    placeholder.overlay(ProgressView())
                }
            }
        } else {
            placeholder
        }
    }

    private var placeholder: some View {
        // Placeholder surface uses the secondary-background token (base01); the
        // glyph on top uses the secondary-text token.
        Rectangle()
            .fill(themeColors.secondaryBackground)
            .overlay(
                Image(systemName: "music.mic")
                    .foregroundStyle(themeColors.secondaryText),
            )
    }
}
