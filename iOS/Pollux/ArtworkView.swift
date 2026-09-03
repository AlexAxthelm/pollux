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
                    .strokeBorder(.quaternary, lineWidth: 0.5),
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
        // The fill and hairline border stay `.quaternary`: a translucent,
        // adaptive `ShapeStyle` with no single base16 equivalent. Only the glyph,
        // which is a real foreground color, is themed.
        Rectangle()
            .fill(.quaternary)
            .overlay(
                Image(systemName: "music.mic")
                    .foregroundStyle(themeColors.secondaryText),
            )
    }
}
