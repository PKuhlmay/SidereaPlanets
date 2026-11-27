import SwiftUI

/// The frame bar of the Alignpoints, Align and Limit steps (docs/design.md, "Frame-Leiste"): status, Go to frame, a slider over
/// all frames, previous/next and the position.
struct FrameBar: View {
    let model: AppModel

    private var aligned: Bool { model.progress.step >= .aligned }

    /// Go to frame counts from 1, the slider and the model from 0.
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
                Circle().fill(SP.ok).frame(width: 9, height: 9).opacity(aligned ? 1 : 0.45)
                Text(aligned ? "Aligned" : "Loaded")
                    .font(.sp(SPMetrics.fontSizeSmall, aligned ? .semibold : .regular))
                    .foregroundStyle(aligned ? SP.ok : SP.muted)
            }
            .frame(minWidth: 76, alignment: .leading)

            HStack(spacing: 8) {
                Text("Go to frame").font(.sp)
                TextField("Go to frame", value: goto, format: .number.grouping(.never))
                    .textFieldStyle(.plain)
                    .multilineTextAlignment(.trailing)
                    .font(.sp)
                    .padding(.horizontal, 6)
                    .frame(width: 56, height: SPMetrics.fieldHeight)
                    .background(SP.field, in: RoundedRectangle(cornerRadius: SPMetrics.radiusSmall))
                    .overlay { RoundedRectangle(cornerRadius: SPMetrics.radiusSmall).strokeBorder(SP.lineStrong, lineWidth: 1) }
                    .accessibilityLabel("Go to frame")
                Stepper("Go to frame", value: goto, in: 1...max(model.frameCount, 1)).labelsHidden()
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
            .buttonStyle(.spBorderless)
            .labelStyle(.iconOnly)

            Slider(value: frame, in: 0...Double(max(model.frameCount - 1, 1)))
                .controlSize(.small)
                .tint(SP.accent)
                .disabled(model.frameCount < 2)
                .accessibilityLabel("Frame")

            Text("Frame \(model.frameIndex + 1) of \(model.frameCount)")
                .font(.sp)
                .frame(minWidth: 118, alignment: .trailing)
        }
        .padding(.horizontal, SPMetrics.panelPadding)
        .frame(height: SPMetrics.frameBarHeight)
        .background(SP.panel)
        .overlay(alignment: .top) { SP.line.frame(height: 1) }
    }
}
