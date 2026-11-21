import App
import SwiftUI

/// The global settings page, reached from the gear in the library toolbar. It edits the
/// library-wide values every subscription follows unless it overrides them. Each row reads
/// from `core.view.settings` and writes with `.setGlobalSetting`; nothing is kept here.
struct SettingsScreen: View {
    @ObservedObject var core: Core
    @Environment(\.themeColors) private var themeColors

    var body: some View {
        Form {
            PlaybackSettingsSection(
                settings: core.view.settings,
                footer: "These apply to every podcast unless it has its own setting.",
                onChange: { core.update(.setGlobalSetting($0)) },
            )
        }
        .scrollContentBackground(.hidden)
        .background(themeColors.background)
        .navigationTitle("Settings")
        .navigationBarTitleDisplayMode(.inline)
    }
}

/// The Playback section of a settings page: skip forward, skip back and rewind on resume.
/// The global page and a subscription's page show the same rows, so the section takes the
/// level's `SettingsView` and reports a change as a `SettingChange` (`nil` resets it).
struct PlaybackSettingsSection: View {
    @Environment(\.themeColors) private var themeColors
    let settings: SettingsView
    let footer: String
    let onChange: (SettingChange) -> Void

    var body: some View {
        Section {
            skipRow("Skip forward", settings.skipForwardSecs, SettingChange.skipForwardSecs)
            skipRow("Skip back", settings.skipBackSecs, SettingChange.skipBackSecs)
            SettingRow(
                title: "Rewind on resume",
                setting: settings.resumeRewindSecs,
                presets: SettingsPresets.rewind,
                zeroLabel: "Off",
                onChange: { onChange(.resumeRewindSecs($0)) },
            )
        } header: {
            Text("Playback")
        } footer: {
            Text(footer)
        }
        .listRowBackground(themeColors.secondaryBackground)
    }

    private func skipRow(
        _ title: String,
        _ setting: SettingValue,
        _ change: @escaping (UInt32?) -> SettingChange,
    ) -> some View {
        SettingRow(
            title: title,
            setting: setting,
            presets: SettingsPresets.skip,
            zeroLabel: nil,
            onChange: { onChange(change($0)) },
        )
    }
}

/// One setting: a picker over the presets, then either a quiet "(default)" when the user
/// hasn't set it or a button that puts it back. `onChange` gets the new value, or `nil` to
/// reset. Choosing the default's own number is a choice, so it counts as set (settings-cascade).
struct SettingRow: View {
    @Environment(\.themeColors) private var themeColors
    let title: String
    let setting: SettingValue
    let presets: [UInt32]
    let zeroLabel: String?
    let onChange: (UInt32?) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Picker(title, selection: selection) {
                ForEach(SettingsPresets.options(presets, including: setting.value), id: \.self) { secs in
                    Text(SettingsPresets.label(secs, zero: zeroLabel)).tag(secs)
                }
            }
            .foregroundStyle(themeColors.text)
            status
        }
    }

    private var selection: Binding<UInt32> {
        Binding(get: { setting.value }, set: { onChange($0) })
    }

    @ViewBuilder private var status: some View {
        if setting.isSet {
            Button("Reset to \(SettingsPresets.label(setting.resetValue, zero: zeroLabel))") {
                onChange(nil)
            }
            // Borderless, or the picker claims the whole list row and swallows the tap.
            .buttonStyle(.borderless)
            .font(.caption)
        } else {
            Text("(default)")
                .font(.caption)
                .foregroundStyle(themeColors.secondaryText)
        }
    }
}

#Preview {
    NavigationStack { SettingsScreen(core: Core()) }
}
