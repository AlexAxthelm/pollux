@testable import Pollux
import Testing

struct SettingsFormattingTests {
    @Test func `labels read in seconds`() {
        #expect(SettingsPresets.label(30) == "30 s")
        #expect(SettingsPresets.label(0) == "0 s")
    }

    @Test func `zero uses its own label when one is given`() {
        #expect(SettingsPresets.label(0, zero: "Off") == "Off")
        #expect(SettingsPresets.label(5, zero: "Off") == "5 s")
    }

    @Test func `presets are offered as they are`() {
        #expect(SettingsPresets.options(SettingsPresets.skip, including: 30) == SettingsPresets.skip)
    }

    @Test func `a value outside the presets is added in order`() {
        #expect(SettingsPresets.options([10, 30, 60], including: 45) == [10, 30, 45, 60])
        #expect(SettingsPresets.options(SettingsPresets.rewind, including: 20) == [0, 3, 5, 10, 20])
    }

    @Test func `the defaults are among the presets`() {
        // The core's defaults (shared/src/defaults.rs) should be choosable without an extra row.
        #expect(SettingsPresets.skip.contains(30))
        #expect(SettingsPresets.skip.contains(15))
        #expect(SettingsPresets.rewind.contains(3))
    }
}
