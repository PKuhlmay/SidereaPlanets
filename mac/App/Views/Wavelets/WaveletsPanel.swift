import AppKit
import SwiftUI
import UniformTypeIdentifiers

// The settings of step 6 in the left panel (docs/design.md, "6 Wavelets"): Colour and LRGB and the six layers first, the rest under
// Advanced. Values, defaults, ranges and locked buttons are those of the Windows app (WaveletsStep.vue); Process and Do All sit in the
// foot of the panel (`StepPanel`).

struct WaveletsStepView: View {
    @Bindable var model: AppModel

    private let columns = (layer: CGFloat(34), field: CGFloat(68), preview: CGFloat(50))

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 24) {
                Toggle(isOn: $model.wavelet.colour) { Text("Colour").font(.rk).foregroundStyle(model.isMono ? RK.off : RK.text) }
                    .toggleStyle(.checkbox)
                    .disabled(model.isMono)
                    .help(model.isMono ? "Needs a colour image" : "Process the channels in colour; off gives grayscale")
                Toggle(isOn: $model.wavelet.lrgb) {
                    Text("LRGB").font(.rk).foregroundStyle(model.isMono || !model.wavelet.colour ? RK.off : RK.text)
                }
                .toggleStyle(.checkbox)
                .disabled(model.isMono || !model.wavelet.colour)
                .help(model.isMono ? "Needs a colour image" : "Sharpen the luminance only; the colour comes from the stack")
            }
            .padding(.bottom, 6)

            HStack {
                GroupHeading(title: "Wavelets")
                Spacer()
                LinkButton("Reset Wavelets") { model.resetWavelets() }
            }
            .padding(.top, 2)
            header
            ForEach(model.wavelet.values.layers.indices, id: \.self) { index in
                layer(index)
            }

            AdvancedSection(model: model, step: .wavelets, count: 8) { advanced }
        }
    }

    // MARK: Layers

    private var header: some View {
        HStack(spacing: 8) {
            Text("Layer").frame(width: columns.layer, alignment: .leading)
            Text("Denoise").frame(width: columns.field, alignment: .leading)
            Text("Sharpen").frame(width: columns.field, alignment: .leading)
            Spacer(minLength: 0)
            Text("Preview").frame(width: columns.preview, alignment: .trailing)
        }
        .font(.rkSmall)
        .foregroundStyle(RK.muted)
        .lineLimit(1)
    }

    /// Two lines: check mark with the number, Denoise, Sharpen and the Preview button with the value of the slider, then the slider.
    private func layer(_ index: Int) -> some View {
        let number = index + 1
        return VStack(spacing: 2) {
            HStack(spacing: 8) {
                Toggle(isOn: $model.wavelet.values.layers[index].enabled) { Text("\(number)").font(.rk).foregroundStyle(RK.text) }
                    .toggleStyle(.checkbox)
                    .frame(width: columns.layer, alignment: .leading)
                    .accessibilityLabel("Layer \(number)")
                DecimalField(
                    label: "Denoise layer \(number)", value: $model.wavelet.values.layers[index].denoise, decimals: 2, step: 0.01,
                    range: WaveletLimits.amount
                )
                DecimalField(
                    label: "Sharpen layer \(number)", value: $model.wavelet.values.layers[index].sharpen, decimals: 3, step: 0.005,
                    range: WaveletLimits.amount
                )
                Spacer(minLength: 0)
                Button(formatPreview(model.wavelet.values.layers[index].level)) {}
                    .buttonStyle(.rkCompact)
                    .disabled(true)
                    .frame(width: columns.preview)
                    .help("Preview layer \(number): Not available yet")
            }
            Slider(
                value: Binding(
                    get: { Double(model.wavelet.values.layers[index].level) },
                    set: { model.moveLevel(index, to: Int($0.rounded())) }
                ),
                in: Double(WaveletLimits.level.lowerBound)...Double(WaveletLimits.level.upperBound), step: 1
            )
            .controlSize(.small)
            .tint(RK.accent)
            .accessibilityLabel("Layer \(number)")
            .accessibilityValue(formatPreview(model.wavelet.values.layers[index].level))
        }
        .padding(.top, 3)
    }

    // MARK: Advanced

    private var advanced: some View {
        VStack(alignment: .leading, spacing: 6) {
            GroupHeading(title: "Wavelets")
            PanelCheckbox(title: "Automatic", isOn: $model.wavelet.automatic)
            PanelCheckbox(title: "Hold Wavelet Setting", isOn: $model.wavelet.hold)
            Text("Waveletscheme").font(.rkSmall).foregroundStyle(RK.muted).padding(.top, 2)
            SegmentBar(items: [
                SegmentItem(title: "Dyadic (2^n)", isOn: model.wavelet.values.scheme == .dyadic) { model.wavelet.values.scheme = .dyadic },
                SegmentItem(title: "Linear", isOn: model.wavelet.values.scheme == .linear) { model.wavelet.values.scheme = .linear },
            ])
            HStack(spacing: 8) {
                Text("Initial Layer").font(.rk).foregroundStyle(RK.text).lineLimit(1)
                NumberField(label: "Initial Layer", value: $model.wavelet.values.initialLayer, range: 1...64, width: 30)
                Text("Step Incr.").font(.rk).foregroundStyle(RK.text).lineLimit(1).help("Step Increment")
                NumberField(label: "Step Increment", value: $model.wavelet.values.stepIncrement, range: 0...64, width: 30)
            }
            .padding(.top, 2)
            Text("Wavelet filter").font(.rkSmall).foregroundStyle(RK.muted).padding(.top, 2)
            SegmentBar(items: [
                SegmentItem(title: "Default", isOn: model.wavelet.values.filter == .default) { model.wavelet.values.filter = .default },
                SegmentItem(title: "Gaussian", isOn: model.wavelet.values.filter == .gaussian) { model.wavelet.values.filter = .gaussian },
            ])
            PanelCheckbox(title: "Use Linked Wavelets", isOn: $model.wavelet.linked).padding(.top, 2)

            RK.line.frame(height: 1).padding(.vertical, 6)
            GroupHeading(title: "Available schemes")
            // The list of schemes has no function yet; Load Scheme and Save Scheme work on a file.
            Picker("Available schemes", selection: .constant(0)) { Text("").tag(0) }
                .labelsHidden()
                .disabled(true)
                .help("Not available yet")
            HStack(spacing: 6) {
                Button("Load Scheme") { Task { await model.chooseSchemeToLoad() } }
                    .buttonStyle(.rkWide)
                    .disabled(model.isBusy)
                Button("Save Scheme") { model.chooseSchemeToSave() }
                    .buttonStyle(.rkWide)
            }
            .padding(.top, 2)

            RK.line.frame(height: 1).padding(.vertical, 6)
            VStack(alignment: .leading, spacing: 2) {
                Button("Realign with Processed") {}
                    .buttonStyle(.rkWide)
                    .disabled(true)
                Text("Not available yet").font(.rkSmall).foregroundStyle(RK.muted)
            }
            .help("Not available yet")
            Button("Stack Again") {}
                .buttonStyle(.rkWide)
                .disabled(true)
        }
    }
}

/// The native dialogs of Load Scheme and Save Scheme: `.rwv` files, the format of the reference program (docs/formats.md).
@MainActor
enum SchemePicker {
    static let message = "the reference program Wavelet scheme settings"
    private static var types: [UTType] { [UTType(filenameExtension: "rwv")].compactMap { $0 } }

    static func chooseToLoad() async -> URL? {
        let panel = NSOpenPanel()
        panel.message = message
        panel.prompt = "Load Scheme"
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = false
        panel.allowedContentTypes = types
        return await panel.begin() == .OK ? panel.url : nil
    }

    static func chooseToSave() async -> URL? {
        let panel = NSSavePanel()
        panel.message = message
        panel.prompt = "Save Scheme"
        panel.nameFieldStringValue = "scheme.rwv"
        panel.allowedContentTypes = types
        return await panel.begin() == .OK ? panel.url : nil
    }
}

extension AppModel {
    /// Load Scheme: asks for a file and puts its values into the panel.
    func chooseSchemeToLoad() async {
        guard !isBusy, let url = await SchemePicker.chooseToLoad() else { return }
        await loadScheme(url)
    }

    /// Save Scheme: asks for a file and writes the values of the panel to it.
    func chooseSchemeToSave() {
        Task {
            guard let url = await SchemePicker.chooseToSave() else { return }
            saveScheme(url)
        }
    }
}
