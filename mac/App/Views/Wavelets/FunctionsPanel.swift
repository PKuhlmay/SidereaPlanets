import SwiftUI

/// The right panel of step 6 (docs/design.md, "6 Wavelets"): the Adjustments buttons, Contrast/Brightness and Remember. Every button
/// is a switch for its own card over the image (`FunctionCards`); the buttons whose card does not exist yet are dimmed. Adjust › at the
/// top of the image area shows and hides this panel.
struct FunctionsPanel: View {
    @Bindable var model: AppModel

    private var hasImage: Bool { model.waveletData.current != nil }
    private var hasCopy: Bool { model.waveletData.clipboard != nil }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                functions
                SP.line.frame(height: 1)
                contrastBrightness
                SP.line.frame(height: 1)
                remember
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(SPMetrics.panelPadding)
        }
        .frame(width: SPMetrics.sideWidthRight)
        .background(SP.panel)
        .overlay(alignment: .leading) { SP.line.frame(width: 1) }
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Adjustments")
    }

    // MARK: Functions

    private var functions: some View {
        VStack(alignment: .leading, spacing: 8) {
            GroupHeading(title: "Adjustments")
            LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: 6), count: 3), spacing: 6) {
                ForEach(functionNames, id: \.self) { name in
                    FunctionButton(title: name, isOn: name == cropFunction ? model.crop.isEditing : model.wavelet.functions.contains(name)) { model.toggleFunction(name) }
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
                // Keep has no effect yet (the state is only stored), so it is locked.
                LinkButton("Keep", isEnabled: false, help: "Not available yet") {}
                LinkButton("Reset") { model.resetContrast() }
            }
            // Each slider has its label and its field right above it.
            HStack {
                Text("Contrast").font(.sp).foregroundStyle(SP.text)
                Spacer()
                NumberField(
                    label: "Contrast",
                    value: Binding(get: { contrastField(model.tone.contrast) }, set: { model.tone.contrast = contrastSlider($0) }),
                    width: 62
                )
            }
            Slider(
                value: Binding(get: { Double(model.tone.contrast) }, set: { model.tone.contrast = Int($0.rounded()) }),
                in: Double(ToneLimits.contrast.lowerBound)...Double(ToneLimits.contrast.upperBound), step: 1
            )
            .controlSize(.small)
            .tint(SP.accent)
            .accessibilityLabel("Contrast")
            HStack {
                Text("Brightness").font(.sp).foregroundStyle(SP.text)
                Spacer()
                NumberField(label: "Brightness", value: $model.tone.brightness, range: ToneLimits.brightness, width: 62)
            }
            .padding(.top, 2)
            Slider(
                value: Binding(get: { Double(model.tone.brightness) }, set: { model.tone.brightness = Int($0.rounded()) }),
                in: Double(ToneLimits.brightness.lowerBound)...Double(ToneLimits.brightness.upperBound), step: 1
            )
            .controlSize(.small)
            .tint(SP.accent)
            .accessibilityLabel("Brightness")
        }
    }

    // MARK: Remember

    private var remember: some View {
        VStack(alignment: .leading, spacing: 6) {
            SegmentBar(items: [
                SegmentItem(title: "Remember", isEnabled: hasImage, help: "Remembers the current image so that it can be compared later.") { model.copyTo() },
                SegmentItem(title: "Restore", isEnabled: hasCopy, help: "Brings the remembered image back.") { model.loadTo() },
                SegmentItem(
                    title: "Difference", isOn: model.waveletData.toneView == .difference, isEnabled: hasCopy,
                    help: "Shows where the current image differs from the remembered one, amplified."
                ) { model.showDifference() },
            ])
            Text("Show").font(.spSmall).foregroundStyle(SP.muted).padding(.top, 2)
            RadioRow(title: "Current image", isSelected: model.waveletData.toneView != .clipboard) { model.showToneView(.current) }
            RadioRow(title: "Remembered image", isSelected: model.waveletData.toneView == .clipboard, isEnabled: hasCopy) {
                model.showToneView(.clipboard)
            }
        }
    }
}

/// An Adjustments button: a switch that is on while its card is open (amber outline and text on `--accent-soft`). The names wrap to two
/// lines; a button without a card is dimmed and locked.
private struct FunctionButton: View {
    let title: String
    let isOn: Bool
    let action: () -> Void
    @State private var isHovering = false

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.sp(SPMetrics.fontSizeSmall, .medium))
                .foregroundStyle(isOn ? SP.accentInk : SP.text)
                .multilineTextAlignment(.center)
                .lineLimit(2)
                .fixedSize(horizontal: false, vertical: true)
                .padding(.horizontal, 3)
                .frame(maxWidth: .infinity, minHeight: 38)
                .background(isOn ? SP.accentSoft : (isHovering ? SP.raisedHover : SP.raised), in: RoundedRectangle(cornerRadius: SPMetrics.radius))
                .overlay { RoundedRectangle(cornerRadius: SPMetrics.radius).strokeBorder(isOn ? SP.accent : SP.lineStrong, lineWidth: 1) }
                .contentShape(RoundedRectangle(cornerRadius: SPMetrics.radius))
        }
        .buttonStyle(.plain)
        .disabled(!isFunctionBuilt(title))
        .opacity(isFunctionBuilt(title) ? 1 : 0.45)
        .onHover { isHovering = $0 }
        .accessibilityAddTraits(isOn ? .isSelected : [])
        .help(functionHelp(title))
    }
}
