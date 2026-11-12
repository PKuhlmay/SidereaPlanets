import SwiftUI

/// The Settings window (⌘,). The Windows app has the CPUs field in its toolbar and the other settings in the Settings menu; on the Mac
/// they belong here (docs/design.md, "Mac-App: bewusste Abweichungen"). The other settings of the Windows menu are not used by any step
/// yet; each one comes with the step that uses it.
struct SettingsView: View {
    @State private var cpus: CpuSettings

    init(model: AppModel) {
        _cpus = State(initialValue: CpuSettings.live(for: model))
    }

    var body: some View {
        Form {
            Section {
                LabeledContent("CPUs") {
                    HStack(spacing: 8) {
                        TextField("CPUs", value: Binding(get: { cpus.value }, set: { cpus.set($0) }), format: .number.grouping(.never))
                            .labelsHidden()
                            .multilineTextAlignment(.trailing)
                            .frame(width: 56)
                        Stepper("CPUs", value: Binding(get: { cpus.value }, set: { cpus.set($0) }), in: 1...cpus.maximum)
                            .labelsHidden()
                    }
                }
                Text(
                    "The number of threads all calculations use, from 1 to \(cpus.maximum) (all cores of this Mac). "
                        + "A change applies to the next calculation; a running one finishes with the number it started with."
                )
                .font(.rkSmall)
                .foregroundStyle(RK.muted)
                .fixedSize(horizontal: false, vertical: true)
                if !cpus.error.isEmpty {
                    Text(cpus.error).font(.rkSmall).foregroundStyle(RK.accentInk)
                }
            }
        }
        .formStyle(.grouped)
        .frame(width: 440)
        .fixedSize(horizontal: false, vertical: true)
        .tint(RK.accent)
    }
}
