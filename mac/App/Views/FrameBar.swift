import SwiftUI

/// The frame bar of the Alignpoints, Align and Limit steps (docs/design.md, "Frame-Leiste"): status, Goto Frame, a slider over
/// all frames, previous/next and the position.
struct FrameBar: View {
    let model: AppModel

    private var aligned: Bool { model.progress.step >= .aligned }

    /// Goto Frame counts from 1, the slider and the model from 0.
    private var goto: Binding<Int> {
        Binding(
            get: { model.frameIndex + 1 },
            set: { model.showFrame(min(max($0, 1), max(model.frameCount, 1)) - 1) }
        )
    }

    private var frame: Binding<Double> {
        Binding(get: { Double(model.frameIndex) }, set: { model.showFrame(Int($0.rounded())) })
    }

    var body: some View {
        HStack(spacing: 14) {
            HStack(spacing: 6) {
                Circle().fill(RK.ok).frame(width: 9, height: 9).opacity(aligned ? 1 : 0.45)
                Text(aligned ? "Aligned" : "Loaded")
                    .font(.rk(RKMetrics.fontSizeSmall, aligned ? .semibold : .regular))
                    .foregroundStyle(aligned ? RK.ok : RK.muted)
            }
            .frame(minWidth: 76, alignment: .leading)

            HStack(spacing: 8) {
                Text("Goto Frame").font(.rk)
                TextField("Goto Frame", value: goto, format: .number.grouping(.never))
                    .textFieldStyle(.plain)
                    .multilineTextAlignment(.trailing)
                    .font(.rk)
                    .padding(.horizontal, 6)
                    .frame(width: 56, height: RKMetrics.fieldHeight)
                    .background(RK.field, in: RoundedRectangle(cornerRadius: RKMetrics.radiusSmall))
                    .overlay { RoundedRectangle(cornerRadius: RKMetrics.radiusSmall).strokeBorder(RK.lineStrong, lineWidth: 1) }
                    .accessibilityLabel("Goto Frame")
                Stepper("Goto Frame", value: goto, in: 1...max(model.frameCount, 1)).labelsHidden()
            }
            .fixedSize()

            HStack(spacing: 2) {
                Button { model.showNextFrame(by: -1) } label: { Image(systemName: "chevron.left") }
                    .help("Previous frame (⌘[)")
                    .disabled(model.frameIndex <= 0)
                Button { model.showNextFrame(by: 1) } label: { Image(systemName: "chevron.right") }
                    .help("Next frame (⌘])")
                    .disabled(model.frameIndex >= model.frameCount - 1)
            }
            .buttonStyle(.rkBorderless)
            .labelStyle(.iconOnly)

            Slider(value: frame, in: 0...Double(max(model.frameCount - 1, 1)))
                .controlSize(.small)
                .tint(RK.accent)
                .disabled(model.frameCount < 2)
                .accessibilityLabel("Frame")

            Text("Frame (\(model.frameIndex + 1)): \(model.frameIndex + 1)/\(model.frameCount)")
                .font(.rk)
                .frame(minWidth: 118, alignment: .trailing)
        }
        .padding(.horizontal, RKMetrics.panelPadding)
        .frame(height: RKMetrics.frameBarHeight)
        .background(RK.panel)
        .overlay(alignment: .top) { RK.line.frame(height: 1) }
    }
}
