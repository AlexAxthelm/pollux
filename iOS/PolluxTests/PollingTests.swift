import Foundation
import Testing

@testable import Pollux

@Suite("Polling")
@MainActor
struct PollingTests {
    private let fast: Duration = .milliseconds(2)

    @Test func returnsImmediatelyWhenNothingIsBusy() async {
        var checks = 0

        await Polling.waitWhileBusy(every: fast) {
            checks += 1
            return false
        }

        #expect(checks == 1, "checked once, found it idle, did not sleep")
    }

    @Test func waitsUntilTheStateGoesIdle() async {
        var checks = 0

        await Polling.waitWhileBusy(every: fast) {
            checks += 1
            return checks <= 3 // busy for the first three looks
        }

        #expect(checks == 4, "re-checked until the fourth look found it idle")
    }

    @Test(.timeLimit(.minutes(1)))
    func cancellationEndsTheWaitEvenWhileStillBusy() async {
        let wait = Task { @MainActor in
            await Polling.waitWhileBusy(every: fast) { true } // never goes idle
        }
        try? await Task.sleep(for: .milliseconds(30))

        wait.cancel()
        await wait.value // would hang past the time limit if cancellation were ignored
    }

    @Test func aTaskCancelledBeforeItStartsDoesNotSleep() async {
        var checks = 0
        let wait = Task { @MainActor in
            await Polling.waitWhileBusy(every: .seconds(60)) { // a sleep would outlast the test
                checks += 1
                return true
            }
        }
        wait.cancel()

        await wait.value

        #expect(checks <= 1, "the cancelled wait returns instead of polling")
    }

    @Test func theDefaultIntervalIsShortEnoughForASpinner() {
        #expect(Polling.interval <= .milliseconds(250))
    }
}
