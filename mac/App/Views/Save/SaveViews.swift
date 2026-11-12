import SidereaPlanetsKit
import SwiftUI

// The pieces of the step Save and of Create AVI (docs/design.md, "7 Save" and "5 Stack"): the content of the step Save, the menu
// of Save image with the file types, and the group Create AVI in the Advanced section of the step Stack. Texts, values and locked
// options are those of the Windows app (SaveStep.vue, SaveImageMenu.vue, StackStep.vue).

/// Step 7 Save: what Save image writes. The action itself is in the foot of the panel.
struct SaveStepView: View {
    let model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("The sharpened stack with the colour and tone settings.")
                .font(.rkSmall)
                .foregroundStyle(RK.muted)
            if !model.progress.waveletsDone {
                Text("Do All runs first with the current values.")
                    .font(.rkSmall)
                    .foregroundStyle(RK.muted)
            }
        }
    }
}

/// Save image: the file type is chosen first (the Stack step offers seven, the Save step five), then the dialog opens. Filled it is the
/// action of the step Save, standard it sits under the result of the step Stack.
struct SaveImageMenu: View {
    let model: AppModel
    let source: ImageSource
    var filled = false

    var body: some View {
        Menu {
            ForEach(model.formats(for: source), id: \.id) { format in
                Button("\(format.label)  .\(format.extension)") {
                    Task { await model.chooseAndSaveImage(format) }
                }
            }
        } label: {
            HStack(spacing: 4) {
                Text("Save image")
                Image(systemName: "chevron.down").font(.system(size: 9, weight: .semibold))
            }
        }
        .menuStyle(.button)
        .buttonStyle(filled ? .rkFilled : .rkWide)
        .disabled(!model.canSaveImage)
        .help("Save the image as a file; choose the file type first")
    }
}

/// The group Create AVI of the Advanced section in the step Stack: Codec, Compress Qual and Framerate, then Save Aligned with its
/// options and Save Integrated with its options.
struct CreateAviSection: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            RK.line.frame(height: 1).padding(.vertical, 4)
            GroupHeading(title: "Create AVI")
            HStack {
                Text("Codec").font(.rk).foregroundStyle(RK.text)
                Spacer(minLength: 4)
                Picker("Codec", selection: $model.stackOptions.codec) {
                    ForEach(aviCodecs, id: \.self) { Text($0).tag($0) }
                }
                .labelsHidden()
                .frame(width: 120)
            }
            wideFieldRow("Compress Qual", value: $model.stackOptions.compressQuality)
            wideFieldRow("Framerate", value: $model.stackOptions.framerate)

            Text("Create aligned Sequence").font(.rkSmall).foregroundStyle(RK.muted).padding(.top, 6)
            VStack(alignment: .leading, spacing: 2) {
                Button("Save Aligned") { Task { await model.chooseAndSaveVideo(.aligned) } }
                    .buttonStyle(.rk)
                    .disabled(!model.canSaveVideo)
                    .help("Write the frames that stay, moved onto the reference frame, as an AVI")
                VStack(alignment: .leading, spacing: 0) {
                    ForEach(AlignedSize.allCases) { size in
                        RadioRow(title: size.label, isSelected: model.stackOptions.alignedSize == size) {
                            model.stackOptions.alignedSize = size
                        }
                    }
                }
            }
            HStack(spacing: 12) {
                PanelCheckbox(title: "Sort On Quality", isOn: $model.stackOptions.sortOnQuality)
                PanelCheckbox(title: "Use Prefilters", isOn: $model.stackOptions.usePrefilters)
            }

            Text("Create integrated sequence").font(.rkSmall).foregroundStyle(RK.muted).padding(.top, 6)
            VStack(alignment: .leading, spacing: 2) {
                Button("Save Integrated") { Task { await model.chooseAndSaveVideo(.integrated) } }
                    .buttonStyle(.rk)
                    .disabled(!model.canSaveVideo)
                    .help("Write the moving average over Average of n Frames aligned frames as an AVI")
                VStack(alignment: .leading, spacing: 0) {
                    RadioRow(title: "Moving average", isSelected: model.stackOptions.integratedMethod == .moving) {
                        model.stackOptions.integratedMethod = .moving
                    }
                    // Locked: only the moving average exists (docs/formats.md, "Create AVI").
                    RadioRow(title: "Compressed", isSelected: false, isEnabled: false) {}
                        .help("Not available yet")
                }
            }
            HStack(spacing: 6) {
                Text("Average of").font(.rk).foregroundStyle(RK.text)
                NumberField(label: "Average of n Frames", value: $model.stackOptions.averageFrames, width: 38)
                Text("Frames").font(.rk).foregroundStyle(RK.text)
            }
        }
    }

    /// A label with a field wide enough for four digits (Compress Qual starts at 7500).
    private func wideFieldRow(_ title: String, value: Binding<Int>) -> some View {
        HStack(spacing: 4) {
            Text(title).font(.rk).foregroundStyle(RK.text).lineLimit(1)
            Spacer(minLength: 0)
            NumberField(label: title, value: value, width: 56)
        }
    }
}
