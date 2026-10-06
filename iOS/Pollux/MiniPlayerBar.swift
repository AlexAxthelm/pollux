import App
import SwiftUI

/// The persistent compact player, shown at the bottom whenever an episode is active
/// and the full player is closed. Exactly two touch zones: play/pause, and everything
/// else (which opens the full player). The progress strip is visual only.
struct MiniPlayerBar: View {
    @Environment(\.themeColors) private var themeColors
    let player: PlayerView
    let onOpen: () -> Void
    let onTogglePlay: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            if let error = player.error {
                NoticeBanner(message: error)
            }
            HStack(spacing: 12) {
                Button(action: onOpen) {
                    HStack(spacing: 12) {
                        ArtworkView(urlString: player.artworkUrl, size: 44)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(player.episodeTitle)
                                .font(.subheadline)
                                .fontWeight(.semibold)
                                .foregroundStyle(themeColors.text)
                                .lineLimit(1)
                            // The active source is the subscription for now; playlists
                            // will make this a distinct "From:" label.
                            Text(player.feedTitle)
                                .font(.caption)
                                .foregroundStyle(themeColors.secondaryText)
                                .lineLimit(1)
                        }
                        Spacer(minLength: 0)
                    }
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .accessibilityLabel("Now playing: \(player.episodeTitle)")
                .accessibilityHint("Opens the player")

                Button(action: onTogglePlay) {
                    Image(systemName: player.isPlaying ? "pause.fill" : "play.fill")
                        .font(.title2)
                        .frame(width: 44, height: 44)
                }
                .buttonStyle(.plain)
                .foregroundStyle(themeColors.text)
                .accessibilityLabel(player.isPlaying ? "Pause" : "Play")
            }
            .padding(.horizontal)
            .padding(.vertical, 8)

            progressStrip
        }
        .background(themeColors.secondaryBackground.ignoresSafeArea(edges: .bottom))
    }

    private var progressStrip: some View {
        GeometryReader { geometry in
            let fraction = PlayerFormatting.fraction(
                position: player.positionSecs, duration: player.durationSecs,
            )
            Rectangle()
                .fill(themeColors.accent)
                .frame(width: geometry.size.width * fraction)
        }
        .frame(height: 2)
        .accessibilityHidden(true)
    }
}
