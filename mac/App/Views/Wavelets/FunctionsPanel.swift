import SwiftUI

/// The right panel of step 6 (docs/design.md, "6 Wavelets"): the Functions buttons, Contrast/Brightness and Copy To. Every Functions
/// button is a switch for its own card over the image (`FunctionCards`); the buttons whose card does not exist yet say so in the status
/// bar. Functions » at the top of the image area shows and hides this panel.
struct FunctionsPanel: View {
    @Bindable var model: AppModel

    private var hasImage: Bool { model.waveletData.current != nil }
    private var hasCopy: Bool { model.waveletData.clipboard != nil }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                functions
                RK.line.frame(height: 1)
                contrastBrightness
                RK.line.frame(height: 1)
                copyTo
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(RKMetrics.panelPadding)
        }
        .frame(width: RKMetrics.sideWidthRight)
        .background(RK.panel)
        .overlay(alignment: .leading) { RK.line.frame(width: 1) }
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Functions")
    }

    // MARK: Functions

    private var functions: some View {
        VStack(alignment: .leading, spacing: 8) {
            GroupHeading(title: "Functions")
            LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 6), count: 3), spacing: 6) {
                ForEach(functionNames, id: \.self) { name in
                    FunctionButton(title: name, isOn: model.wavelet.functions.contains(name)) { model.toggleFunction(name) }
                }
            }
        }
    }

    // MARK: Contrast/Brightness

    private var contrastBrightness: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 0) {
                GroupHeading(title: "Contrast/Brightness")
                Spacer()
                LinkButton("Hold", isOn: model.tone.holdContrast) { model.tone.holdContrast.toggle() }
                LinkButton("Reset") { model.resetContrast() }
            }
            Slider(
                value: Binding(get: { Double(model.tone.contrast) }, set: { model.tone.contrast = Int($0.rounded()) }),
                in: Double(ToneLimits.contrast.lowerBound)...Double(ToneLimits.contrast.upperBound), step: 1
            )
            .controlSize(.small)
            .tint(RK.accent)
            .accessibilityLabel("Contrast")
            Slider(
                value: Binding(get: { Double(model.tone.brightness) }, set: { model.tone.brightness = Int($0.rounded()) }),
                in: Double(ToneLimits.brightness.lowerBound)...Double(ToneLimits.brightness.upperBound), step: 1
            )
            .controlSize(.small)
            .tint(RK.accent)
            .accessibilityLabel("Brightness")
            HStack(alignment: .top, spacing: 12) {
                VStack(alignment: .leading, spacing: 3) {
                    Text("Contrast").font(.rk).foregroundStyle(RK.text)
                    NumberField(
                        label: "Contrast",
                        value: Binding(get: { contrastField(model.tone.contrast) }, set: { model.tone.contrast = contrastSlider($0) }),
                        width: 62
                    )
                }
                VStack(alignment: .leading, spacing: 3) {
                    Text("Brightness").font(.rk).foregroundStyle(RK.text)
                    NumberField(label: "Brightness", value: $model.tone.brightness, range: ToneLimits.brightness, width: 62)
                }
            }
            .padding(.top, 2)
        }
    }

    // MARK: Copy To

    private var copyTo: some View {
        VStack(alignment: .leading, spacing: 6) {
            SegmentBar(items: [
                SegmentItem(title: "Copy To", isEnabled: hasImage) { model.copyTo() },
                SegmentItem(title: "Load to", isEnabled: hasCopy) { model.loadTo() },
                SegmentItem(title: "Difference", isOn: model.waveletData.toneView == .difference, isEnabled: hasCopy) { model.showDifference() },
            ])
            Text("Toggle").font(.rkSmall).foregroundStyle(RK.muted).padding(.top, 2)
            RadioRow(title: "Current Image", isSelected: model.waveletData.toneView != .clipboard) { model.showToneView(.current) }
            RadioRow(title: "Clipboard Image", isSelected: model.waveletData.toneView == .clipboard, isEnabled: hasCopy) {
                model.showToneView(.clipboard)
            }
        }
    }
}

/// A Functions button: a switch that is on while its card is open (amber outline and text on `--accent-soft`). The names wrap to two lines.
private struct FunctionButton: View {
    let title: String
    let isOn: Bool
    let action: () -> Void
    @State private var isHovering = false

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.rk(RKMetrics.fontSizeSmall, .medium))
                .foregroundStyle(isOn ? RK.accentInk : RK.text)
                .multilineTextAlignment(.center)
                .lineLimit(2)
                .fixedSize(horizontal: false, vertical: true)
                .padding(.horizontal, 3)
                .frame(maxWidth: .infinity, minHeight: 38)
                .background(isOn ? RK.accentSoft : (isHovering ? RK.raisedHover : RK.raised), in: RoundedRectangle(cornerRadius: RKMetrics.radius))
                .overlay { RoundedRectangle(cornerRadius: RKMetrics.radius).strokeBorder(isOn ? RK.accent : RK.lineStrong, lineWidth: 1) }
                .contentShape(RoundedRectangle(cornerRadius: RKMetrics.radius))
        }
        .buttonStyle(.plain)
        .onHover { isHovering = $0 }
        .accessibilityAddTraits(isOn ? .isSelected : [])
        .help(isToneCard(title) ? "" : "Not available yet")
    }
}
