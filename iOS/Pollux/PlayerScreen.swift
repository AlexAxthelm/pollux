import App
import AVKit
import SwiftUI

/// The full player: a paged content area (artwork, show-notes placeholder), scrubber,
/// transport controls, and the source/option rows. Presented as a full-screen cover
/// from the root; "Hide" dismisses it back to the mini-player.
struct PlayerScreen: View {
    @Environment(\.themeColors) private var themeColors
    let player: PlayerView
    let send: (Event) -> Void
    let onHide: () -> Void
    let onGoToSource: () -> Void

    /// While the thumb is dragged, the value shown (the core's position keeps ticking
    /// underneath and must not fight the drag). Committed as one seek on release.
    @State private var scrubPosition: Double?
    @State private var showRemaining = false
    @State private var page = 0

    var body: some View {
        VStack(spacing: 16) {
            pages
            if let error = player.error {
                NoticeBanner(message: error)
            }
            scrubber
            titleBlock
            transport
            bottomRow
        }
        .padding()
        .background(themeColors.background.ignoresSafeArea())
    }

    // MARK: - Paged content

    private var pages: some View {
        VStack(spacing: 8) {
            TabView(selection: $page) {
                ArtworkView(urlString: player.artworkUrl, size: 300)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .tag(0)
                Text("Show notes coming soon")
                    .foregroundStyle(themeColors.secondaryText)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .tag(1)
            }
            .tabViewStyle(.page(indexDisplayMode: .never))
            // The system dots are fixed light-on-light; draw our own in theme colors.
            HStack(spacing: 8) {
                ForEach(0 ..< 2, id: \.self) { index in
                    Circle()
                        .fill(index == page ? themeColors.text : themeColors.secondaryText.opacity(0.4))
                        .frame(width: 8, height: 8)
                }
            }
            .accessibilityHidden(true)
        }
    }

    // MARK: - Scrubber

    private var duration: UInt32? {
        player.durationSecs
    }

    private var displayedPosition: Double {
        scrubPosition ?? Double(player.positionSecs)
    }

    private var scrubber: some View {
        VStack(spacing: 4) {
            Slider(
                value: Binding(
                    get: { displayedPosition },
                    set: { scrubPosition = $0 },
                ),
                in: 0 ... Double(max(duration ?? 1, 1)),
                onEditingChanged: { editing in
                    guard !editing, let target = scrubPosition else { return }
                    scrubPosition = nil
                    send(.seekTo(UInt32(target)))
                },
            )
            .disabled(duration == nil)
            .accessibilityLabel("Playback position")
            .accessibilityValue(PlayerFormatting.spoken(UInt32(displayedPosition)))
            HStack {
                Text(PlayerFormatting.clock(UInt32(displayedPosition)))
                Spacer()
                Button {
                    showRemaining.toggle()
                } label: {
                    Text(trailingTime)
                }
                .buttonStyle(.plain)
                .accessibilityLabel(showRemaining ? "Time remaining" : "Total duration")
                .accessibilityHint("Toggles between total and remaining time")
            }
            .font(.caption)
            .monospacedDigit()
            .foregroundStyle(themeColors.secondaryText)
        }
    }

    private var trailingTime: String {
        guard let duration else { return "--:--" }
        return showRemaining
            ? PlayerFormatting.remaining(position: UInt32(displayedPosition), duration: duration)
            : PlayerFormatting.clock(duration)
    }

    // MARK: - Title and source

    private var titleBlock: some View {
        VStack(spacing: 4) {
            Text(player.episodeTitle)
                .font(.headline)
                .foregroundStyle(themeColors.text)
                .multilineTextAlignment(.center)
                .lineLimit(2)
            Button(action: onGoToSource) {
                Text("From: \(player.feedTitle)")
                    .font(.subheadline)
                    .foregroundStyle(themeColors.accent)
                    .lineLimit(1)
            }
            .accessibilityHint("Goes to the podcast")
        }
    }

    // MARK: - Transport

    private var transport: some View {
        HStack(spacing: 40) {
            skipButton(forward: false)
            Button { send(.togglePlay) } label: {
                Image(systemName: player.isPlaying ? "pause.circle.fill" : "play.circle.fill")
                    .font(.system(size: 64))
            }
            .accessibilityLabel(player.isPlaying ? "Pause" : "Play")
            skipButton(forward: true)
        }
        .foregroundStyle(themeColors.text)
    }

    private func skipButton(forward: Bool) -> some View {
        let seconds = forward ? player.skipForwardSecs : player.skipBackSecs
        return Button { send(forward ? .skipForward : .skipBack) } label: {
            VStack(spacing: 2) {
                Image(systemName: forward ? "goforward" : "gobackward")
                    .font(.title)
                Text(PlayerFormatting.skipLabel(seconds, forward: forward))
                    .font(.caption)
                    .monospacedDigit()
            }
        }
        .accessibilityLabel("Skip \(forward ? "forward" : "back") \(seconds) seconds")
    }

    // MARK: - Bottom row

    private var bottomRow: some View {
        HStack {
            Button(action: onHide) {
                Image(systemName: "chevron.down")
                    .font(.title3)
                    .frame(width: 44, height: 44)
            }
            .accessibilityLabel("Hide player")
            Spacer()
            playerOptions
            Spacer()
            episodeOptions
        }
        .foregroundStyle(themeColors.text)
    }

    /// Output routing plus the not-yet-built options, shown disabled so the menu's
    /// shape is visible without pretending they work.
    private var playerOptions: some View {
        HStack(spacing: 8) {
            RoutePicker()
                .frame(width: 44, height: 44)
            Menu {
                Button("Playback speed", systemImage: "speedometer") {}.disabled(true)
                Button("Sleep timer", systemImage: "moon.zzz") {}.disabled(true)
                Button("Equalizer", systemImage: "slider.vertical.3") {}.disabled(true)
            } label: {
                Image(systemName: "slider.horizontal.3")
                    .font(.title3)
                    .frame(width: 44, height: 44)
            }
            .accessibilityLabel("Player options")
        }
    }

    private var episodeOptions: some View {
        Menu {
            Button("Go to podcast", systemImage: "books.vertical", action: onGoToSource)
        } label: {
            Image(systemName: "ellipsis.circle")
                .font(.title3)
                .frame(width: 44, height: 44)
        }
        .accessibilityLabel("Episode options")
    }
}

/// The system AirPlay / output-route picker.
private struct RoutePicker: UIViewRepresentable {
    @Environment(\.themeColors) private var themeColors

    func makeUIView(context _: Context) -> AVRoutePickerView {
        let picker = AVRoutePickerView()
        picker.prioritizesVideoDevices = false
        return picker
    }

    func updateUIView(_ picker: AVRoutePickerView, context _: Context) {
        picker.tintColor = UIColor(themeColors.text)
        picker.activeTintColor = UIColor(themeColors.accent)
    }
}
