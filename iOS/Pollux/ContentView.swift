import App
import SwiftUI

struct ContentView: View {
    @ObservedObject var core: Core

    var body: some View {
        NavigationStack {
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
            .navigationTitle("Library")
        }
    }
}

#Preview {
    ContentView(core: Core())
}
