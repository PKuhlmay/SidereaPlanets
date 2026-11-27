import AppKit
import SwiftUI

/// CPU load and memory of the app, for the left end of the status bar; the same readout as Siderea's. Plain input: the values come from
/// `ResourceMonitor`, a missing value shows a dash. When the bar is too narrow the readout drops the memory first, then everything; it
/// never wraps or overflows.
struct ResourceReadout: View {
    let cpuPercent: Double?
    let memoryBytes: Int64?

    /// The processor cores the CPU value is a share of.
    private var coreCount: Int { ProcessInfo.processInfo.activeProcessorCount }

    var body: some View {
        ViewThatFits(in: .horizontal) {
            items(memory: true)
            items(memory: false)
            Color.clear.frame(width: 0, height: 0)
        }
    }

    private func items(memory: Bool) -> some View {
        HStack(spacing: 14) {
            // The number of cores in brackets says what 100 % stands for.
            item(
                "CPU (\(coreCount))", spoken: "CPU load", value: ProcessUsage.cpuReading(cpuPercent, coreCount: coreCount),
                help: "CPU load of Siderea Planets as a share of all \(coreCount) processor cores. 100 % means every core is fully busy.",
                highlighted: ProcessUsage.isHigh(cpuPercent: cpuPercent, coreCount: coreCount))
            if memory {
                item(
                    "RAM", spoken: "Memory", value: memoryBytes.map { ProcessUsage.byteText($0) },
                    help: "Memory Siderea Planets currently holds.")
            }
        }
        .lineLimit(1)
        .fixedSize(horizontal: true, vertical: false)
    }

    private func item(_ label: String, spoken: String, value: String?, help: String, highlighted: Bool = false) -> some View {
        HStack(spacing: 5) {
            Text(verbatim: label)
            Text(verbatim: value ?? "–")
                .foregroundStyle(highlighted ? SP.accentInk : SP.text)
                // The accent colour is not the only cue for a high load: it is also bold.
                .fontWeight(highlighted ? .bold : .regular)
                .frame(minWidth: 50, alignment: .trailing)
        }
        .help(help)
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(spoken)
        // The tooltip says what the number is; VoiceOver gets it as the hint.
        .accessibilityHint(help)
        .accessibilityValue(
            (value.map { $0.trimmingCharacters(in: .whitespaces) } ?? "not available")
                // The accent colour alone marks a high load; VoiceOver hears it too.
                + (highlighted ? ", high load" : ""))
    }
}

/// The readout as the first field of the status bar: switched on in Settings, measuring only while the window is on screen (a
/// measurement costs a little). Draws nothing while it is off.
struct ResourceField: View {
    @Environment(\.scenePhase) private var scenePhase
    @AppStorage(ResourceSettings.key) private var isShown = ResourceSettings.defaultIsShown
    @State private var monitor = ResourceMonitor()
    /// Whether any of the app's windows is on screen at all; a minimized window or a hidden app is not, and macOS does not always report
    /// that as a background scene phase.
    @State private var isOnScreen = true

    var body: some View {
        Group {
            if isShown {
                ResourceReadout(cpuPercent: monitor.cpuPercent, memoryBytes: monitor.memoryBytes)
                    .padding(.horizontal, 12)
                    .frame(maxHeight: .infinity)
                    .overlay(alignment: .trailing) { SP.line.frame(width: 1) }
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: NSWindow.didChangeOcclusionStateNotification)) { _ in
            isOnScreen = NSApp.windows.contains { $0.isVisible && $0.occlusionState.contains(.visible) }
        }
        .onChange(of: ResourceSettings.isMeasuring(isShown: isShown, isBackground: scenePhase == .background, isOnScreen: isOnScreen), initial: true) {
            _, isMeasuring in
            if isMeasuring { monitor.start() } else { monitor.stop() }
        }
        .onDisappear { monitor.stop() }
    }
}
