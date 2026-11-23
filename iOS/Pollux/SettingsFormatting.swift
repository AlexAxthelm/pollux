import Foundation

/// The choices the Settings screen offers for the playback settings, and how a value reads.
/// Kept out of the views so it can be unit-tested without SwiftUI (like `PlayerFormatting`).
/// The core accepts any whole number of seconds; these are only what the picker shows.
enum SettingsPresets {
    /// Seconds the skip buttons can jump.
    static let skip: [UInt32] = [10, 15, 30, 45, 60, 90]
    /// Seconds playback rewinds on resume. Zero is "Off".
    static let rewind: [UInt32] = [0, 3, 5, 10]

    /// `presets` plus `current` when it isn't one of them (a value set by hand or by another
    /// build), in order, so the picker can always show what is in effect.
    static func options(_ presets: [UInt32], including current: UInt32) -> [UInt32] {
        Array(Set(presets).union([current])).sorted()
    }

    /// "30 s", or `zero` for a value of 0 when one is given ("Off").
    static func label(_ seconds: UInt32, zero: String? = nil) -> String {
        if seconds == 0, let zero {
            return zero
        }
        return "\(seconds) s"
    }
}
