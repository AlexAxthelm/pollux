import Combine
import Foundation

/// Counts how often a published value changes after the point of subscription (the
/// initial value, which `@Published` emits on subscribe, is not counted). Lives in its own
/// file because importing Combine makes `Subscription` ambiguous with the app's own type.
@MainActor
final class Publishes {
    private(set) var count = 0
    private var subscription: AnyCancellable?

    init(_ publisher: some Publisher<some Any, Never>) {
        subscription = publisher.dropFirst().sink { [unowned self] _ in count += 1 }
    }
}
