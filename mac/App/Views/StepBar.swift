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
                            .fill(model.stepStates[step.previous ?? .open]?.status == .done ? RK.ok : RK.lineStrong)
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
                    .font(.rkSmall)
                    .foregroundStyle(RK.muted)
                    .lineLimit(1)
                    .truncationMode(.tail)
                    .help(details.stepBarLine)
            }
        }
        .padding(.horizontal, RKMetrics.panelPadding)
        .frame(height: RKMetrics.stepBarHeight)
        .background(RK.panel)
        .overlay(alignment: .bottom) { RK.line.frame(height: 1) }
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
                    .font(.rk(RKMetrics.fontSize, isShown ? .semibold : .regular))
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
            if !isFilledCircle { Circle().strokeBorder(isLocked ? RK.line : RK.lineStrong, lineWidth: 1) }
            if state.status == .done {
                Image(systemName: "checkmark").font(.system(size: 9, weight: .bold)).foregroundStyle(RK.onAccent)
            } else {
                Text("\(step.number)")
                    .font(.rk(11, .semibold))
                    .foregroundStyle(isShown ? RK.onAccent : (isLocked ? RK.off : RK.muted))
            }
        }
        .frame(width: 18, height: 18)
    }

    private var isFilledCircle: Bool { isShown || state.status == .done }

    private var circleFill: Color {
        if state.status == .done { return RK.ok }
        return isShown ? RK.accent : .clear
    }

    private var textColor: Color {
        if isLocked { return RK.off }
        return isShown ? RK.accentInk : RK.text
    }

    private var fill: Color {
        if isShown { return RK.accentSoft }
        return isHovering && isClickable ? RK.raisedHover : .clear
    }

    /// A locked pill names the step that is missing; a done pill names its result.
    private var tooltip: String {
        if isLocked { return state.hint }
        if state.status == .done, !result.isEmpty { return "\(step.label) · \(result)" }
        return ""
    }
}
