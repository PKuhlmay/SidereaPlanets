import SwiftUI

/// The setting that shows or hides the CPU and memory readout in the status bar (saved with `@AppStorage`).
nonisolated enum ResourceSettings {
    static let key = "showResourceUsage"
    /// On by default, as in Siderea.
    static let defaultIsShown = true

    /// Measuring costs a little, so it runs only while the readout is shown and the window is on screen.
    static func isMeasuring(isShown: Bool, isBackground: Bool, isOnScreen: Bool) -> Bool {
        isShown && !isBackground && isOnScreen
    }
}

/// The switch in the Settings window.
struct ResourceSettingsSection: View {
    @AppStorage(ResourceSettings.key) private var isShown = ResourceSettings.defaultIsShown

    var body: some View {
        Section("Status Bar") {
            Toggle("Show CPU and memory usage", isOn: $isShown)
            Text("CPU load of this app as a share of all cores and the memory it holds, at the left end of the status bar.")
                .font(.rkSmall)
                .foregroundStyle(RK.muted)
                .fixedSize(horizontal: false, vertical: true)
        }
    }
}
