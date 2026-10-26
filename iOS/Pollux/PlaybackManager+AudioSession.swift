import AVFoundation
import Foundation
import os

/// Releasing the audio session. Split from `PlaybackManager` to keep that type's file under the
/// length limit; the properties it uses are internal for that reason.
extension PlaybackManager {
    /// Hands the audio session back so other apps (music the listener paused to play this) are
    /// told they may resume. The system often refuses while the item that was just removed
    /// still has I/O winding down, and a dropped refusal would leave the session held for good,
    /// so a refused release is tried again shortly, a few times. Claiming the session again
    /// (the listener started something) cancels any pending try.
    func releaseAudioSession() {
        deactivationTask?.cancel()
        deactivationTask = nil
        guard !tryDeactivating() else { return }
        deactivationTask = Task { @MainActor [weak self] in
            for _ in 1 ..< Self.deactivationAttempts {
                guard let self else { return }
                try? await Task.sleep(for: deactivationRetryDelay)
                if Task.isCancelled {
                    return
                }
                if tryDeactivating() {
                    return
                }
            }
            Self.log.error("Couldn't release the audio session after \(Self.deactivationAttempts) tries")
        }
    }

    private func tryDeactivating() -> Bool {
        do {
            try deactivateAudioSession()
            return true
        } catch {
            return false
        }
    }
}
