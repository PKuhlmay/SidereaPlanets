import SwiftUI

/// A 3 x 3 mixer behind the Color Mixing button: every output channel is the sum of red, green and blue times three weights, in percent.
/// Neutral is 100 on the diagonal. It also works on gray images, where all three input channels are equal.
struct ColorMixingCard: View {
    @Bindable var model: AppModel

    private let channels = ["Red", "Green", "Blue"]

    var body: some View {
        FloatCard(title: "Color Mixing", close: { model.closeFunction("Color Mixing") }, actions: {
            LinkButton("Reset", isEnabled: !model.tone.mix.isIdentity) { model.tone.mix.matrix = ColorMixValues.identityMatrix }
        }, content: {
            VStack(alignment: .leading, spacing: 8) {
                presetMenu
                ForEach(0..<3, id: \.self) { output in
                    VStack(spacing: 3) {
                        Text("\(channels[output]) output").font(.rkSmall).foregroundStyle(RK.muted)
                            .frame(maxWidth: .infinity, alignment: .leading)
                        ForEach(0..<3, id: \.self) { input in weight(output: output, input: input) }
                    }
                }
                PanelCheckbox(
                    title: "Preserve brightness", isOn: $model.tone.mix.preserveBrightness,
                    help: "Keeps the brightness of every pixel while the colors change."
                )
            }
            .padding(10)
        })
    }

    private var presetMenu: some View {
        HStack {
            Text("Preset").font(.rk).foregroundStyle(RK.text)
            Spacer()
            Menu {
                ForEach(mixPresets) { preset in
                    Button(preset.name) { model.tone.mix.matrix = preset.matrix }
                }
            } label: {
                HStack(spacing: 5) {
                    Text("Choose").font(.rk)
                    Image(systemName: "chevron.down").font(.system(size: 9, weight: .semibold))
                }
                .foregroundStyle(RK.text)
                .padding(.horizontal, 10)
                .frame(height: RKMetrics.fieldHeight)
                .background(RK.raised, in: RoundedRectangle(cornerRadius: RKMetrics.radius))
                .overlay { RoundedRectangle(cornerRadius: RKMetrics.radius).strokeBorder(RK.lineStrong, lineWidth: 1) }
            }
            .menuStyle(.button)
            .buttonStyle(.plain)
            .menuIndicator(.hidden)
            .fixedSize()
            .help("Sets all nine weights at once")
        }
    }

    private func weight(output: Int, input: Int) -> some View {
        let name = "\(channels[input]) in \(channels[output].lowercased()) output"
        return HStack(spacing: 8) {
            Text(channels[input]).font(.rk).foregroundStyle(RK.text).frame(width: 40, alignment: .leading)
            Slider(
                value: Binding(get: { Double(model.tone.mix[output, input]) }, set: { model.tone.mix[output, input] = Int($0.rounded()) }),
                in: Double(ToneLimits.mix.lowerBound)...Double(ToneLimits.mix.upperBound), step: 1
            )
            .controlSize(.small)
            .tint(RK.accent)
            .accessibilityLabel(name)
            NumberField(
                label: "\(name) percent",
                value: Binding(get: { model.tone.mix[output, input] }, set: { model.tone.mix[output, input] = $0 }),
                range: ToneLimits.mix, width: 36
            )
            Text("%").font(.rkSmall).foregroundStyle(RK.muted)
        }
        .frame(minHeight: 26)
    }
}
