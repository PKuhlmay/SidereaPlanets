import SidereaPlanetsKit
import SwiftUI

// The cards Denoise/Deringing and Masking (docs/design.md, "Karten hinter den Functions-Schaltflächen"; the algorithms are in
// docs/wavelets.md). Both start from the sharpened image and the stack it was made from, so they only change anything after Stack.

/// A label with its field above a slider, in whole numbers.
private struct RefineSlider: View {
    let title: String
    /// What the field is called for VoiceOver, e.g. "Denoise percent".
    let fieldLabel: String
    @Binding var value: Int
    let range: ClosedRange<Int>
    var help = ""

    var body: some View {
        VStack(spacing: 2) {
            HStack(spacing: 6) {
                Text(title).font(.rk).foregroundStyle(RK.text)
                Spacer()
                NumberField(label: fieldLabel, value: $value, range: range, width: 40)
            }
            Slider(
                value: Binding(get: { Double(value) }, set: { value = Int($0.rounded()) }),
                in: Double(range.lowerBound)...Double(range.upperBound), step: 1
            )
            .controlSize(.small)
            .tint(RK.accent)
            .accessibilityLabel(title)
        }
        .help(help)
    }
}

/// Denoise (edge-preserving smoothing) and Deringing (lifts the dark ring next to bright edges), each with its strength in percent.
struct DenoiseCard: View {
    @Bindable var model: AppModel

    var body: some View {
        FloatCard(title: "Denoise/Deringing", close: { model.closeFunction("Denoise/Deringing") }, actions: {
            LinkButton("Reset") { model.tone.refine.resetDenoise() }
        }, content: {
            VStack(alignment: .leading, spacing: 8) {
                RefineSlider(
                    title: "Denoise (%)", fieldLabel: "Denoise percent", value: $model.tone.refine.denoise, range: RefineLimits.percent,
                    help: "How strongly noise is smoothed. Edges, such as the edge of the planet, stay sharp."
                )
                RefineSlider(
                    title: "Deringing (%)", fieldLabel: "Deringing percent", value: $model.tone.refine.deringing, range: RefineLimits.percent,
                    help: "How much of the dark ring next to bright edges is lifted back to the level of the original stack."
                )
                RefineSlider(
                    title: "Radius (px)", fieldLabel: "Deringing radius in pixels", value: $model.tone.refine.radius, range: RefineLimits.radius,
                    help: "How far from an edge Deringing looks for the dark ring."
                )
                Text("Deringing compares the sharpened image with the stack")
                    .font(.rkSmall).foregroundStyle(RK.muted)
                    .fixedSize(horizontal: false, vertical: true)
            }
            .padding(10)
        })
    }
}

/// Limits the sharpening to the bright part of the stack (the planet): a threshold, a soft edge, invert and a preview of the mask.
struct MaskCard: View {
    @Bindable var model: AppModel

    var body: some View {
        FloatCard(title: "Masking", close: { model.closeFunction("Masking") }, actions: {
            LinkButton("Reset") { model.tone.refine.resetMask() }
        }, content: {
            VStack(alignment: .leading, spacing: 8) {
                PanelCheckbox(
                    title: "Use mask", isOn: $model.tone.refine.maskOn,
                    help: "Sharpens only where the stack is brighter than the threshold. Everywhere else the unsharpened stack stays."
                )
                RefineSlider(
                    title: "Threshold", fieldLabel: "Threshold", value: $model.tone.refine.threshold, range: RefineLimits.threshold,
                    help: "Brightness of the stack, from 0 (black) to 255 (white), above which the image is sharpened."
                )
                RefineSlider(
                    title: "Feather (px)", fieldLabel: "Feather in pixels", value: $model.tone.refine.feather, range: RefineLimits.feather,
                    help: "Width of the soft edge between the sharpened and the unsharpened part."
                )
                HStack(spacing: 14) {
                    PanelCheckbox(title: "Invert", isOn: $model.tone.refine.invert, help: "Sharpens the dark part instead of the bright part.")
                    PanelCheckbox(
                        title: "Show mask", isOn: $model.tone.refine.showMask,
                        help: "Tints the part that is sharpened red. This only changes what you see here, not the image."
                    )
                }
            }
            .padding(10)
        })
    }
}
