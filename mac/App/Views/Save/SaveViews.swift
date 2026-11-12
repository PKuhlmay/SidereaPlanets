import SidereaPlanetsKit
import SwiftUI

// The pieces of the step Save and of Create AVI (docs/design.md, "7 Save" and "5 Stack"): the content of the step Save, the menu
// of Save image with the file types, and the group Create AVI in the Advanced section of the step Stack. Texts, values and locked
// options are those of the Windows app (SaveStep.vue, SaveImageMenu.vue, StackStep.vue).

/// Step 7 Save: what Save image writes. The action itself is in the foot of the panel.
struct SaveStepView: View {
    let model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Saves the sharpened stack with your color and tone settings.")
                    .font(.rkSmall)
                    .foregroundStyle(RK.muted)
                if !model.progress.waveletsDone {
                    Text("Your latest changes are applied to the whole image first (Do All).")
                        .font(.rkSmall)
                        .foregroundStyle(RK.muted)
                }
            }
            RK.line.frame(height: 1)
            CropSection(model: model)
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
                Button("\(format.label) · .\(format.extension)…") {
                    Task { await model.chooseAndSaveImage(format) }
                }
            }
        } label: {
            HStack(spacing: 4) {
                Text("Save Image")
                Image(systemName: "chevron.down").font(.system(size: 9, weight: .semibold))
            }
        }
        .menuStyle(.button)
        .buttonStyle(filled ? .rkFilled : .rkWide)
        .disabled(!model.canSaveImage)
        .help("Saves the image as a file; choose the file type first.")
    }
}

/// The group Create AVI of the Advanced section in the step Stack: Codec, Compression quality and Frame rate, then Save Aligned Video
/// with its options and Save Averaged Video with its options.
struct CreateAviSection: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            RK.line.frame(height: 1).padding(.vertical, 4)
            GroupHeading(title: "Create AVI")
                .help("Saves the aligned frames as a video, for example for animations or other software.")
            // Locked: the video is always written uncompressed.
            HStack {
                Text("Codec").font(.rk).foregroundStyle(RK.off)
                Spacer(minLength: 4)
                Picker("Codec", selection: $model.stackOptions.codec) {
                    ForEach(aviCodecs, id: \.self) { Text($0).tag($0) }
                }
                .labelsHidden()
                .disabled(true)
                .opacity(0.45)
                .frame(width: 120)
            }
            .help("Not available yet. The video is always uncompressed.")
            wideFieldRow("Compression quality", value: $model.stackOptions.compressQuality, isLocked: true)
            wideFieldRow("Frame rate (fps)", value: $model.stackOptions.framerate)

            Text("Aligned video").font(.rkSmall).foregroundStyle(RK.muted).padding(.top, 6)
            VStack(alignment: .leading, spacing: 2) {
                Button("Save Aligned Video…") { Task { await model.chooseAndSaveVideo(.aligned) } }
                    .buttonStyle(.rk)
                    .disabled(!model.canSaveVideo)
                    .help("Saves the kept frames, aligned, as an AVI video.")
                Text("Frame size").font(.rkSmall).foregroundStyle(RK.muted).padding(.top, 2)
                VStack(alignment: .leading, spacing: 0) {
                    // Only the original size is written (docs/formats.md, "Create AVI"), so it is always the one that is on.
                    RadioRow(title: AlignedSize.original.label, isSelected: true) {}
                    ForEach(AlignedSize.allCases.filter { $0 != .original }) { size in
                        LockedOption(title: size.label, kind: .radio)
                    }
                }
            }
            HStack(spacing: 12) {
                PanelCheckbox(title: "Sort by quality", isOn: $model.stackOptions.sortOnQuality, help: "Puts the sharpest frames first in the video.")
                PanelCheckbox(title: "Use prefilters", isOn: .constant(false), isLocked: true)
            }

            Text("Averaged video").font(.rkSmall).foregroundStyle(RK.muted).padding(.top, 6)
            VStack(alignment: .leading, spacing: 2) {
                Button("Save Averaged Video…") { Task { await model.chooseAndSaveVideo(.integrated) } }
                    .buttonStyle(.rk)
                    .disabled(!model.canSaveVideo)
                    .help("Saves a video in which each frame is the average of several aligned frames in a row; the number is set below.")
                VStack(alignment: .leading, spacing: 0) {
                    RadioRow(
                        title: "Moving average", isSelected: model.stackOptions.integratedMethod == .moving,
                        help: "Each video frame is the average of the next n aligned frames."
                    ) {
                        model.stackOptions.integratedMethod = .moving
                    }
                    // Locked: only the moving average exists (docs/formats.md, "Create AVI").
                    LockedOption(title: "Compressed", kind: .radio)
                }
            }
            HStack(spacing: 6) {
                Text("Average of").font(.rk).foregroundStyle(RK.text)
                NumberField(label: "Average of n frames", value: $model.stackOptions.averageFrames, width: 38)
                Text("frames").font(.rk).foregroundStyle(RK.text)
            }
        }
    }

    /// A label with a field wide enough for four digits (Compression quality starts at 7500).
    private func wideFieldRow(_ title: String, value: Binding<Int>, isLocked: Bool = false) -> some View {
        HStack(spacing: 4) {
            Text(title).font(.rk).foregroundStyle(isLocked ? RK.off : RK.text).lineLimit(1)
            Spacer(minLength: 0)
            NumberField(label: title, value: value, width: 56)
                .disabled(isLocked)
                .opacity(isLocked ? 0.45 : 1)
        }
        .help(isLocked ? "Not available yet. The video is always uncompressed." : "")
    }
}
