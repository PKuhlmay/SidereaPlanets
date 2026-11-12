import SidereaPlanetsKit
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
                    DerotateOptions(model: model)
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

/// De-rotate images with its settings (MAC-020): turns every frame back to the middle of the recording before it is stacked, so that the details
/// towards the edge of the disc do not smear. Planet and Start time are set by the app from the file when it can; Camera angle is for a camera
/// that was not turned to put north up. The line at the bottom says how far the planet turns over the kept frames.
struct DerotateOptions: View {
    @Bindable var model: AppModel

    /// The Start time field: local time of the Mac, stored as the UTC instant. Empty until the file or the user gives a time.
    private var startTime: Binding<Date> {
        Binding(get: { model.stackOptions.startTime ?? Date() }, set: { model.stackOptions.startTime = $0 })
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            PanelCheckbox(
                title: "De-rotate images", isOn: $model.stackOptions.derotate,
                help: "Turns every frame back to the middle of the recording by the rotation of the planet, so that details at the edge of the disc stay sharp. Needs the date and time of the recording."
            )
            if model.stackOptions.derotate { settings }
        }
    }

    private var settings: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Planet").font(.rkSmall).foregroundStyle(RK.muted)
            SegmentBar(items: [(DerotatePlanet.jupiter, "Jupiter"), (.saturn, "Saturn"), (.mars, "Mars")].map { planet, title in
                SegmentItem(
                    title: title, isOn: model.stackOptions.derotatePlanet == planet,
                    help: "The planet in the recording: it decides how fast it turns and how flattened it is."
                ) { model.stackOptions.derotatePlanet = planet }
            })
            if !model.hasFrameTimestamps { startTimeRow }
            HStack(spacing: 4) {
                Text("Camera angle (°)").font(.rk).foregroundStyle(RK.text)
                Spacer(minLength: 0)
                DecimalField(
                    label: "Camera angle", value: $model.stackOptions.cameraAngle, decimals: 1, step: 1, range: -360...360
                )
            }
            .help("How far the camera was turned, in degrees counter-clockwise on the screen. 0 means celestial north is up in the picture.")
            PanelCheckbox(
                title: "Mirrored image", isOn: $model.stackOptions.mirrored,
                help: "The picture is a mirror image, as a single mirror in the light path makes it: east is on the right when north is up."
            )
            if let line = model.derotationLine { summary(line) }
            if model.hasFrameTimestamps {
                Text("The file has a time for every frame.").font(.rkSmall).foregroundStyle(RK.muted)
            }
        }
        .padding(.leading, 18)
    }

    private var startTimeRow: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 4) {
                Text("Start time").font(.rk).foregroundStyle(RK.text)
                Spacer(minLength: 0)
                DatePicker("Start time", selection: startTime, displayedComponents: [.date, .hourAndMinute])
                    .datePickerStyle(.field)
                    .labelsHidden()
                    .font(.rk)
            }
            .help("When the first frame was taken, in the time zone of this Mac. The file has no time for its frames.")
            if let date = model.stackOptions.startTime {
                Text("\(utcText(date))\(model.startTimeIsGuess ? " · taken from the file date, check it" : "")")
                    .font(.rkSmall).foregroundStyle(RK.muted)
                    .fixedSize(horizontal: false, vertical: true)
            } else {
                Text("Enter the time of the first frame").font(.rkSmall).foregroundStyle(RK.muted)
            }
        }
    }

    private func summary(_ line: (text: String, hint: String?, isProblem: Bool)) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(line.text).font(.rkSmall).foregroundStyle(line.isProblem ? RK.accentInk : RK.text)
            if let hint = line.hint { Text(hint).font(.rkSmall).foregroundStyle(RK.muted) }
        }
        .accessibilityElement(children: .combine)
    }
}
