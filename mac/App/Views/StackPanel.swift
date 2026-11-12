import SwiftUI

// The settings of step 5 in the left panel (docs/design.md, "5 Stack"): Output size, Use nearest points and Correct geometry
// first, the rest of the Stacking options under Advanced. Values and defaults are those of the Windows app (StackStep.vue); the group
// Create AVI with Save Aligned Video and Save Averaged Video is in Views/Save/SaveViews.swift.

struct StackStepView: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Output size").font(.rkSmall).foregroundStyle(RK.muted).help("Size of the stacked image.")
            RadioRow(title: "Default", isSelected: true) {}
            LockedOption(title: "Smallest common area", kind: .radio, hint: "Only the area that every frame covers.")
            LockedOption(title: "Largest area", kind: .radio, hint: "Everything that any frame covers.")
            LockedOption(title: "Region", kind: .radio, hint: "Only the region of interest.")
            FieldRow(
                title: "Use nearest points", value: $model.stackOptions.nearestPoints, range: 1...1_000_000,
                help: "How many of the nearest alignpoints decide how a pixel is shifted: more is smoother, fewer is more local."
            )
            .padding(.top, 6)
            PanelCheckbox(
                title: "Correct geometry", isOn: $model.stackOptions.correctGeometry,
                help: "Shifts each part of the image by its own movement; off moves every frame as a whole."
            )
            .padding(.top, 2)

            AdvancedSection(model: model, step: .stack, count: 14) {
                VStack(alignment: .leading, spacing: 4) {
                    GroupHeading(title: "Stacking options")
                    PanelCheckbox(
                        title: "Even out frame brightness", isOn: $model.stackOptions.normalize,
                        help: "Levels the brightness of the frames before stacking; helps when clouds pass by."
                    )
                    LockedOption(title: "Stretch histogram after stacking", kind: .checkbox, hint: "Spreads the brightness over the full range after stacking.")
                    LockedOption(title: "De-rotate images", kind: .checkbox, hint: "Turns the frames back against the rotation of the planet.")
                    FieldRow(
                        title: "Ignore edge pixels (px)", value: $model.stackOptions.edgePixels,
                        help: "Pixels at the edge of each frame that are left out of the stack, in pixels."
                    )
                    .padding(.vertical, 2)
                    // Locked: the stack does not enlarge the result yet (docs/stacking.md).
                    LockedOption(title: "Resampling", kind: .checkbox, hint: "Enlarges the result.")
                    LockedOption(title: "Drizzling", kind: .checkbox, hint: "Enlarges the result by combining shifted frames.")
                    CreateAviSection(model: model)
                }
            }
        }
    }
}

/// An option the core does not read yet: dimmed, with the tooltip "Not available yet" and the same words at the right of the row,
/// below it (right-aligned) when the name leaves no room.
struct LockedOption: View {
    enum Kind { case radio, checkbox }

    let title: String
    let kind: Kind
    /// What the option would do, in the tooltip after "Not available yet".
    var hint = ""

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
        .help(hint.isEmpty ? "Not available yet" : "Not available yet. \(hint)")
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
