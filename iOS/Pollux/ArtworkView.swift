import SwiftUI

/// Square podcast/episode artwork loaded from a URL, with a neutral placeholder
/// while loading or when the URL is missing or fails.
///
/// With a `size` it is a fixed square. With `size: nil` it fills the width it is
/// offered, as large a square as fits (the full player's artwork).
struct ArtworkView: View {
    @Environment(\.themeColors) private var themeColors
    let urlString: String?
    var size: CGFloat? = 56

    private var cornerRadius: CGFloat {
        size == nil ? 12 : 8
    }

    var body: some View {
        square
            .clipShape(RoundedRectangle(cornerRadius: cornerRadius))
            .overlay(
                RoundedRectangle(cornerRadius: cornerRadius)
                    .strokeBorder(themeColors.secondaryBackground, lineWidth: 0.5),
            )
    }

    @ViewBuilder private var square: some View {
        if let size {
            artwork.frame(width: size, height: size)
        } else {
            // Clear square sets the bounds (largest that fits); the image fills it.
            Color.clear
                .aspectRatio(1, contentMode: .fit)
                .overlay { artwork }
        }
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
