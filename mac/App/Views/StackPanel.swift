import SwiftUI

// The settings of step 5 in the left panel (docs/design.md, "5 Stack"): StackSize, Use Nearestby Alignpoints and Correct geometry
// first, the rest of the Stacking options under Advanced. Values, defaults and locked options are those of the Windows app
// (StackStep.vue); the group Create AVI with Save Aligned and Save Integrated comes with MAC-009.

struct StackStepView: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("StackSize").font(.rkSmall).foregroundStyle(RK.muted)
            RadioRow(title: "Default", isSelected: true) {}
            LockedOption(title: "Minimized", kind: .radio)
            LockedOption(title: "Maximized", kind: .radio)
            LockedOption(title: "R.of Interest", kind: .radio)
            FieldRow(title: "Use Nearestby Alignpoints", value: $model.stackOptions.nearestPoints, range: 1...1_000_000)
                .padding(.top, 6)
            PanelCheckbox(title: "Correct geometry", isOn: $model.stackOptions.correctGeometry)
                .padding(.top, 2)

            AdvancedSection(model: model, step: .stack, count: 6) {
                VStack(alignment: .leading, spacing: 4) {
                    GroupHeading(title: "Stacking options")
                    PanelCheckbox(title: "Normalization of frame intensity", isOn: $model.stackOptions.normalize)
                    LockedOption(title: "Stretch histogram after stacking", kind: .checkbox)
                    LockedOption(title: "De-rotate images", kind: .checkbox)
                    HStack(alignment: .center, spacing: 8) {
                        Text("Do not stack pixels\nat the frameedge").font(.rk).foregroundStyle(RK.text)
                        Spacer(minLength: 4)
                        NumberField(label: "Do not stack pixels at the frameedge", value: $model.stackOptions.edgePixels)
                    }
                    .padding(.vertical, 2)
                    // Locked: the stack does not enlarge the result yet (docs/stacking.md).
                    LockedOption(title: "Resampling", kind: .checkbox)
                    LockedOption(title: "Drizzling", kind: .checkbox)
                }
            }
        }
    }
}

/// An option the core does not read yet: dimmed, with the tooltip "Not available yet" and the same words at the right of the row,
/// below it (right-aligned) when the name leaves no room.
private struct LockedOption: View {
    enum Kind { case radio, checkbox }

    let title: String
    let kind: Kind

    private let note = Text("Not available yet").font(.rkSmall).foregroundStyle(RK.muted)

    var body: some View {
        ViewThatFits(in: .horizontal) {
            HStack(spacing: 6) {
                option
                Spacer(minLength: 4)
                note
            }
            VStack(alignment: .leading, spacing: 0) {
                option
                note.frame(maxWidth: .infinity, alignment: .trailing)
            }
        }
        .help("Not available yet")
        .accessibilityElement(children: .combine)
        .accessibilityHint("Not available yet")
    }

    @ViewBuilder
    private var option: some View {
        switch kind {
        case .radio: RadioRow(title: title, isSelected: false, isEnabled: false) {}
        case .checkbox: PanelCheckbox(title: title, isOn: .constant(false), isLocked: true)
        }
    }
}
