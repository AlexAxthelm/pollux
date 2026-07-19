import Foundation

/// Interprets a change in the core's `loading` flag for an in-flight subscribe.
///
/// The core does not currently report whether a subscribe succeeded, so the
/// shell derives it from a `loading` edge plus the presence of an error. That
/// derivation lives here rather than inline in the view so it can be tested
/// without SwiftUI; the view keeps only the state and the field update.
enum SubscribeFlow {
    enum Outcome: Equatable {
        /// Nothing submitted, or the submission is still running.
        case pending
        /// The submission finished and the feed was saved.
        case succeeded
        /// The submission finished with an error.
        case failed
    }

    static func outcome(isSubscribing: Bool, isLoading: Bool, error: String?) -> Outcome {
        guard isSubscribing, !isLoading else { return .pending }
        return error == nil ? .succeeded : .failed
    }
}
