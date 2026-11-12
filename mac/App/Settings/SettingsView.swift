import SwiftUI

/// The Settings window (⌘,). The Windows app has the CPUs field in its toolbar and the other settings in the Settings menu; on the Mac
/// they belong here (docs/design.md, "Mac-App: bewusste Abweichungen"). The other settings of the Windows menu are not used by any step
/// yet; each one comes with the step that uses it.
struct SettingsView: View {
    @State private var cpus: CpuSettings
    @Bindable private var model: AppModel

    init(model: AppModel) {
        self.model = model
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
            Section("Wavelets") {
                Picker("Processing Area", selection: $model.wavelet.processingAreaSide) {
                    ForEach([2048, 1024, 512, 256, 128], id: \.self) { side in Text("\(side) x \(side)").tag(side) }
                }
                Text("The square in the middle of the image the live preview of the Wavelets step calculates.")
                    .font(.rkSmall)
                    .foregroundStyle(RK.muted)
                    .fixedSize(horizontal: false, vertical: true)
                LabeledContent("Schemes") {
                    HStack(spacing: 8) {
                        Button("Load Scheme…") { Task { await model.chooseSchemeToLoad() } }
                            .disabled(model.isBusy)
                        Button("Save Scheme…") { model.chooseSchemeToSave() }
                    }
                }
                Text("A scheme is a .rwv file with the scheme, filter, Initial Layer, Step Incr. and the six layers of the Wavelets step, the format of the reference program.")
                    .font(.rkSmall)
                    .foregroundStyle(RK.muted)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .formStyle(.grouped)
        .frame(width: 440)
        .fixedSize(horizontal: false, vertical: true)
        .tint(RK.accent)
    }
}
