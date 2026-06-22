import App
import SwiftUI

struct ContentView: View {
    @ObservedObject var core: Core
    @State private var feedUrl = ""

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                Group {
                    if core.view.library.loading {
                        ProgressView()
                    } else if core.view.library.subscriptions.isEmpty {
                        Text("No podcasts yet")
                            .foregroundStyle(.secondary)
                    } else {
                        List(core.view.library.subscriptions, id: \.id) { sub in
                            Text(sub.title)
                        }
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)

                if let error = core.view.library.error {
                    Text(error)
                        .foregroundStyle(.red)
                        .font(.caption)
                        .padding(.horizontal)
                }

                HStack {
                    TextField("Feed URL", text: $feedUrl)
                        .textFieldStyle(.roundedBorder)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                    Button("Add") {
                        core.update(.fetchFeed(feedUrl))
                        feedUrl = ""
                    }
                    .disabled(feedUrl.isEmpty || core.view.library.loading)
                }
                .padding()
            }
            .navigationTitle("Library")
        }
    }
}

#Preview {
    ContentView(core: Core())
}
