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
                LabeledContent("CPU cores to use") {
                    HStack(spacing: 8) {
                        TextField("CPU cores to use", value: Binding(get: { cpus.value }, set: { cpus.set($0) }), format: .number.grouping(.never))
                            .labelsHidden()
                            .multilineTextAlignment(.trailing)
                            .frame(width: 56)
                        Stepper("CPU cores to use", value: Binding(get: { cpus.value }, set: { cpus.set($0) }), in: 1...cpus.maximum)
                            .labelsHidden()
                    }
                }
                Text(
                    "How many CPU cores all calculations use, from 1 to \(cpus.maximum) (all cores of this Mac). "
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
                Picker("Preview area", selection: $model.wavelet.processingAreaSide) {
                    ForEach([2048, 1024, 512, 256, 128], id: \.self) { side in Text("\(side) × \(side) px").tag(side) }
                }
                Text("Size of the square in the middle of the image that the Wavelets preview calculates. Smaller is faster.")
                    .font(.rkSmall)
                    .foregroundStyle(RK.muted)
                    .fixedSize(horizontal: false, vertical: true)
                LabeledContent("Wavelet schemes") {
                    HStack(spacing: 8) {
                        Button("Load Scheme…") { Task { await model.chooseSchemeToLoad() } }
                            .disabled(model.isBusy)
                        Button("Save Scheme…") { model.chooseSchemeToSave() }
                    }
                }
                Text("A wavelet scheme is a .rwv file with the layer spacing, filter, first layer size, size step and the six layers of the Wavelets step.")
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
