import AppKit
import SwiftUI
import UniformTypeIdentifiers

// The settings of step 6 in the left panel (docs/design.md, "6 Wavelets"): Color and Sharpen brightness only and the six layers first, the
// rest under Advanced. Values, defaults and ranges are those of the Windows app (WaveletsStep.vue); Update Preview and Do All sit in the
// foot of the panel (`StepPanel`).

struct WaveletsStepView: View {
    @Bindable var model: AppModel

    private let columns = (layer: CGFloat(34), field: CGFloat(68), preview: CGFloat(50))

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 16) {
                Toggle(isOn: $model.wavelet.colour) { Text("Color").font(.rk).foregroundStyle(model.isMono ? RK.off : RK.text) }
                    .toggleStyle(.checkbox)
                    .disabled(model.isMono)
                    .help(model.isMono ? "Needs a color image" : "Processes the channels in color; off gives grayscale.")
                Toggle(isOn: $model.wavelet.lrgb) {
                    Text("Sharpen brightness only").font(.rk).foregroundStyle(model.isMono || !model.wavelet.colour ? RK.off : RK.text)
                }
                .toggleStyle(.checkbox)
                .disabled(model.isMono || !model.wavelet.colour)
                .help(model.isMono ? "Needs a color image" : "Sharpens only the brightness of the stack and keeps its colors.")
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
                .help("Layer 1 holds the finest detail, higher layers coarser structures; switch a layer off to leave it out.")
            Text("Denoise").frame(width: columns.field, alignment: .leading).help("Smooths the noise in this layer.")
            Text("Sharpen").frame(width: columns.field, alignment: .leading).help("Fine adjustment of this layer; the slider sets the main strength.")
            Spacer(minLength: 0)
            Text("Preview").frame(width: columns.preview, alignment: .trailing)
        }
        .font(.rkSmall)
        .foregroundStyle(RK.muted)
        .lineLimit(1)
    }

    /// Two lines: check mark with the number, Denoise, Sharpen and the locked Preview button, then the slider with its value.
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
                Button { } label: { Image(systemName: "eye") }
                    .buttonStyle(.rkCompact)
                    .disabled(true)
                    .frame(width: columns.preview)
                    .help("Preview layer \(number): Not available yet")
                    .accessibilityLabel("Preview layer \(number)")
            }
            HStack(spacing: 8) {
                Slider(
                    value: Binding(
                        get: { Double(model.wavelet.values.layers[index].level) },
                        set: { model.moveLevel(index, to: Int($0.rounded())) }
                    ),
                    in: Double(WaveletLimits.level.lowerBound)...Double(WaveletLimits.level.upperBound), step: 1
                )
                .controlSize(.small)
                .tint(RK.accent)
                .help("Strength of layer \(number)")
                .accessibilityLabel("Layer \(number)")
                .accessibilityValue(formatPreview(model.wavelet.values.layers[index].level))
                Text(formatPreview(model.wavelet.values.layers[index].level))
                    .font(.rkSmall)
                    .foregroundStyle(RK.muted)
                    .frame(width: columns.preview, alignment: .trailing)
            }
        }
        .padding(.top, 3)
    }

    // MARK: Advanced

    private var advanced: some View {
        VStack(alignment: .leading, spacing: 6) {
            GroupHeading(title: "Wavelet Settings")
            PanelCheckbox(
                title: "Run Do All automatically", isOn: $model.wavelet.automatic,
                help: "Applies the sharpening to the whole image half a second after each change."
            )
            PanelCheckbox(
                title: "Keep layer values for a new stack", isOn: $model.wavelet.hold,
                help: "Keeps the layer values when a new stack is made."
            )
            Text("Layer spacing")
                .font(.rkSmall)
                .foregroundStyle(RK.muted)
                .padding(.top, 2)
                .help("Linear: the layers grow evenly. Dyadic: each layer doubles in size.")
            SegmentBar(items: [
                SegmentItem(
                    title: "Dyadic (2^n)", isOn: model.wavelet.values.scheme == .dyadic,
                    help: "Each layer doubles in size."
                ) { model.wavelet.values.scheme = .dyadic },
                SegmentItem(
                    title: "Linear", isOn: model.wavelet.values.scheme == .linear,
                    help: "The layers grow evenly."
                ) { model.wavelet.values.scheme = .linear },
            ])
            VStack(alignment: .leading, spacing: 4) {
                FieldRow(
                    title: "First layer size", value: $model.wavelet.values.initialLayer, range: 1...64,
                    help: "Size of the finest layer."
                )
                FieldRow(
                    title: "Size step", value: $model.wavelet.values.stepIncrement, range: 0...64,
                    help: "How much each further layer grows."
                )
            }
            .padding(.top, 2)
            Text("Wavelet filter").font(.rkSmall).foregroundStyle(RK.muted).padding(.top, 2)
            SegmentBar(items: [
                SegmentItem(title: "Default", isOn: model.wavelet.values.filter == .default, help: "The standard filter.") { model.wavelet.values.filter = .default },
                SegmentItem(title: "Gaussian", isOn: model.wavelet.values.filter == .gaussian, help: "Smoother than Default.") { model.wavelet.values.filter = .gaussian },
            ])
            PanelCheckbox(
                title: "Move all sliders together", isOn: $model.wavelet.linked,
                help: "Moving one slider moves all the others by the same amount."
            )
            .padding(.top, 2)

            RK.line.frame(height: 1).padding(.vertical, 6)
            GroupHeading(title: "Saved Schemes")
            // The list of saved schemes has no function yet; Load Scheme and Save Scheme work on a file.
            Picker("Saved schemes", selection: .constant(0)) { Text("").tag(0) }
                .labelsHidden()
                .disabled(true)
                .help("Not available yet")
            HStack(spacing: 6) {
                Button("Load Scheme…") { Task { await model.chooseSchemeToLoad() } }
                    .buttonStyle(.rkWide)
                    .disabled(model.isBusy)
                    .help("Loads the layer values from a wavelet scheme file (.rwv).")
                Button("Save Scheme…") { model.chooseSchemeToSave() }
                    .buttonStyle(.rkWide)
                    .help("Saves the layer values as a wavelet scheme file (.rwv).")
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
                .help("Not available yet")
        }
    }
}

/// The native dialogs of Load Scheme and Save Scheme: wavelet scheme files (`.rwv`, docs/formats.md).
@MainActor
enum SchemePicker {
    static let message = "Wavelet scheme"
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
    /// Load Scheme…: asks for a file and puts its values into the panel.
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
