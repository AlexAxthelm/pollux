import App
import SwiftUI

struct ContentView: View {
    @ObservedObject var core: Core
    @Environment(\.themeColors) private var themeColors
    @State private var feedUrl = ""
    @State private var isSubscribing = false
    private var trimmedFeedUrl: String {
        feedUrl.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                Group {
                    if core.view.library.loading {
                        ProgressView()
                    } else if core.view.library.subscriptions.isEmpty {
                        Text("No podcasts yet")
                            .foregroundStyle(themeColors.secondaryText)
                    } else {
                        List(core.view.library.subscriptions, id: \.id) { sub in
                            NavigationLink(value: sub) {
                                Text(sub.title)
                            }
                        }
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)

                if let error = core.view.library.error {
                    Text(error)
                        .foregroundStyle(themeColors.error)
                        .font(.caption)
                        .padding(.horizontal)
                }

                HStack {
                    TextField("Feed URL", text: $feedUrl)
                        .textFieldStyle(.roundedBorder)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                    Button("Add") {
                        isSubscribing = true
                        core.update(.fetchFeed(trimmedFeedUrl))
                    }
                    .disabled(trimmedFeedUrl.isEmpty || core.view.library.loading)
                }
                .padding()
            }
            .navigationTitle("Library")
            .navigationDestination(for: SubscriptionSummary.self) { sub in
                SubscriptionDetailScreen(core: core, subscription: sub)
            }
            .onChange(of: core.view.library.loading) { _, isLoading in
                // Keep the typed URL until a subscribe actually succeeds, so a
                // failed attempt can be corrected instead of retyped.
                switch SubscribeFlow.outcome(
                    isSubscribing: isSubscribing,
                    isLoading: isLoading,
                    error: core.view.library.error,
                ) {
                case .pending:
                    break
                case .succeeded:
                    isSubscribing = false
                    feedUrl = ""
                case .failed:
                    isSubscribing = false
                }
            }
        }
    }
}

#Preview {
    ContentView(core: Core())
}
