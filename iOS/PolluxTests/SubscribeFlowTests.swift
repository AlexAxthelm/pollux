import Foundation
import Testing

@testable import Pollux

@Suite("SubscribeFlow")
struct SubscribeFlowTests {

    // MARK: In flight

    @Test func stillLoadingIsPending() {
        #expect(
            SubscribeFlow.outcome(isSubscribing: true, isLoading: true, error: nil)
                == .pending
        )
    }

    @Test func stillLoadingWithStaleErrorIsPending() {
        #expect(
            SubscribeFlow.outcome(isSubscribing: true, isLoading: true, error: "previous failure")
                == .pending
        )
    }

    // MARK: Completion

    @Test func finishedWithoutErrorSucceeds() {
        #expect(
            SubscribeFlow.outcome(isSubscribing: true, isLoading: false, error: nil)
                == .succeeded
        )
    }

    @Test func finishedWithErrorFails() {
        #expect(
            SubscribeFlow.outcome(
                isSubscribing: true,
                isLoading: false,
                error: "App Transport Security policy requires the use of a secure connection",
            ) == .failed
        )
    }

    // MARK: Loading changes unrelated to subscribing

    /// `loading` also toggles for the initial library load, which must not
    /// clear a URL the user is part-way through typing.
    @Test func notSubscribingIsPendingRegardlessOfLoading() {
        #expect(
            SubscribeFlow.outcome(isSubscribing: false, isLoading: false, error: nil)
                == .pending
        )
        #expect(
            SubscribeFlow.outcome(isSubscribing: false, isLoading: true, error: nil)
                == .pending
        )
    }

    @Test func notSubscribingIsPendingEvenWhenLoadFails() {
        #expect(
            SubscribeFlow.outcome(isSubscribing: false, isLoading: false, error: "db unavailable")
                == .pending
        )
    }
}
