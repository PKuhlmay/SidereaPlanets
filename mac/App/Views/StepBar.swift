import SwiftUI

/// The step bar under the toolbar (docs/design.md, "Schrittleiste"): the seven steps as pills, the details of the recording
/// at the right. Done and possible steps open in the left panel; nothing opens during a calculation.
struct StepBar: View {
    let model: AppModel

    var body: some View {
        HStack(spacing: 16) {
            HStack(spacing: 0) {
                ForEach(ViewStep.allCases) { step in
                    if step != .open {
                        Rectangle()
                            .fill(model.stepStates[step.previous ?? .open]?.status == .done ? SP.ok : SP.lineStrong)
                            .frame(width: 14, height: 1)
                            .padding(.horizontal, 4)
                    }
                    StepPill(
                        step: step,
                        state: model.stepStates[step] ?? StepState(status: .locked, hint: ""),
                        isShown: model.viewStep == step,
                        isBusy: model.isBusy,
                        result: model.result(of: step)
                    ) { model.show(step) }
                }
            }
            .fixedSize()
            Spacer(minLength: 0)
            if let details = model.details {
                Text(details.stepBarLine)
                    .font(.spSmall)
                    .foregroundStyle(SP.muted)
                    .lineLimit(1)
                    .truncationMode(.tail)
                    .help(details.stepBarLine)
            }
        }
        .padding(.horizontal, SPMetrics.panelPadding)
        .frame(height: SPMetrics.stepBarHeight)
        .background(SP.panel)
        .overlay(alignment: .bottom) { SP.line.frame(height: 1) }
    }
}

/// One pill: a numbered circle and the name. Shown: amber fill. Done: green circle with a check. Locked: grey, no click.
private struct StepPill: View {
    let step: ViewStep
    let state: StepState
    let isShown: Bool
    let isBusy: Bool
    let result: String
    let action: () -> Void
    @State private var isHovering = false

    private var isLocked: Bool { state.status == .locked }
    private var isClickable: Bool { !isLocked && !isBusy }

    var body: some View {
        Button(action: action) {
            HStack(spacing: 6) {
                circle
                Text(step.label)
                    .font(.sp(SPMetrics.fontSize, isShown ? .semibold : .regular))
                    .foregroundStyle(textColor)
            }
            .padding(.leading, 4)
            .padding(.trailing, 10)
            .frame(height: 28)
            .background(fill, in: Capsule())
            .contentShape(Capsule())
        }
        .buttonStyle(.plain)
        .disabled(!isClickable)
        .onHover { isHovering = $0 }
        .help(tooltip)
        .accessibilityLabel("\(step.label), step \(step.number) of 7")
        .accessibilityValue(isLocked ? "locked" : (state.status == .done ? "done" : "available"))
        .accessibilityAddTraits(isShown ? .isSelected : [])
    }

    private var circle: some View {
        ZStack {
            Circle().fill(circleFill)
            if !isFilledCircle { Circle().strokeBorder(isLocked ? SP.line : SP.lineStrong, lineWidth: 1) }
            if state.status == .done {
                Image(systemName: "checkmark").font(.system(size: 9, weight: .bold)).foregroundStyle(SP.onAccent)
            } else {
                Text("\(step.number)")
                    .font(.sp(11, .semibold))
                    .foregroundStyle(isShown ? SP.onAccent : (isLocked ? SP.off : SP.muted))
            }
        }
        .frame(width: 18, height: 18)
    }

    private var isFilledCircle: Bool { isShown || state.status == .done }

    private var circleFill: Color {
        if state.status == .done { return SP.ok }
        return isShown ? SP.accent : .clear
    }

    private var textColor: Color {
        if isLocked { return SP.off }
        return isShown ? SP.accentInk : SP.text
    }

    private var fill: Color {
        if isShown { return SP.accentSoft }
        return isHovering && isClickable ? SP.raisedHover : .clear
    }

    /// What the step is; a locked pill adds the step that is missing, a done pill its result.
    private var tooltip: String {
        if isLocked { return "\(step.explanation)\n\(state.hint)" }
        if state.status == .done, !result.isEmpty { return "\(step.explanation)\n\(step.label) · \(result)" }
        return step.explanation
    }
}
