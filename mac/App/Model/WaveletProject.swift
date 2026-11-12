import Foundation
import SidereaPlanetsKit

// The sections `wavelet` and `colour` of the project file, in the layout of the Windows app (`buildProject` of projectfile.ts: a copy of
// `settings.wavelet`, and `{colour, lrgb}`), so a project written by one app opens in the other (docs/formats.md, "Projektdatei").
// Read the way `mergeKnown` does: a key counts only if it is there and has the type of the setting.

extension SectionReader {
    /// A finite number as it is (the fields of Denoise, Sharpen, Clip, Gamma and the shifts have decimals).
    nonisolated func double(_ key: String, _ target: inout Double) {
        if case .number(let value)? = values[key], value.isFinite { target = value }
    }
}

private nonisolated func rounded(_ value: Double, decimals: Int) -> Double {
    let scale = pow(10, Double(decimals))
    return (value * scale).rounded() / scale
}

private nonisolated func name(_ scheme: WaveletScheme) -> String { scheme == .dyadic ? "dyadic" : "linear" }
private nonisolated func name(_ filter: WaveletFilter) -> String { filter == .default ? "default" : "gaussian" }

/// The section `wavelet`: all settings of the Wavelet tab, the Functions cards and Contrast/Brightness.
nonisolated func waveletSection(_ options: WaveletOptions, tone: ToneValues) -> JSON {
    func shift(_ s: ChannelShift) -> JSON { .object(["x": .number(Double(s.x)), "y": .number(Double(s.y))]) }
    let layers = options.values.layers.map { layer -> JSON in
        .object([
            "enabled": .bool(layer.enabled),
            "denoise": .number(rounded(layer.denoise, decimals: 2)),
            "sharpen": .number(rounded(layer.sharpen, decimals: 3)),
            "level": .number(Double(layer.level)),
        ])
    }
    return .object([
        "showFullImage": .bool(options.showFullImage),
        "showAlignpoints": .bool(options.showAlignpoints),
        "showProcessingArea": .bool(options.showProcessingArea),
        "automatic": .bool(options.automatic),
        "hold": .bool(options.hold),
        "scheme": .string(name(options.values.scheme)),
        "initialLayer": .number(Double(options.values.initialLayer)),
        "stepIncrement": .number(Double(options.values.stepIncrement)),
        "filter": .string(name(options.values.filter)),
        "linked": .bool(options.linked),
        "layers": .array(layers),
        "functionsOpen": .bool(options.functionsOpen),
        "functions": .array(options.functions.map { .string($0) }),
        "contrast": .number(Double(tone.contrast)),
        "brightness": .number(Double(tone.brightness)),
        "holdContrast": .bool(tone.holdContrast),
        "tone": .object([
            "shifts": .object(["red": shift(tone.shifts.red), "green": shift(tone.shifts.green), "blue": shift(tone.shifts.blue)]),
            "maxShift": .number(Double(tone.maxShift)),
            "red": .number(Double(tone.red)),
            "green": .number(Double(tone.green)),
            "blue": .number(Double(tone.blue)),
            "black": .number(Double(tone.black)),
            "white": .number(Double(tone.white)),
            "clip": .number(tone.clip),
            "histogramView": .string(tone.histogramView == .input ? "input" : "result"),
            "gamma": .number(tone.gamma),
        ]),
    ])
}

nonisolated func applyWaveletSection(_ section: JSON?, to options: inout WaveletOptions, tone: inout ToneValues) {
    let read = SectionReader(section)
    read.bool("showFullImage", &options.showFullImage)
    read.bool("showAlignpoints", &options.showAlignpoints)
    read.bool("showProcessingArea", &options.showProcessingArea)
    read.bool("automatic", &options.automatic)
    read.bool("hold", &options.hold)
    if case .string(let value)? = read.values["scheme"] {
        if value == "dyadic" { options.values.scheme = .dyadic } else if value == "linear" { options.values.scheme = .linear }
    }
    read.int("initialLayer", &options.values.initialLayer)
    read.int("stepIncrement", &options.values.stepIncrement)
    if case .string(let value)? = read.values["filter"] {
        if value == "default" { options.values.filter = .default } else if value == "gaussian" { options.values.filter = .gaussian }
    }
    read.bool("linked", &options.linked)
    if case .array(let items)? = read.values["layers"] {
        for (index, item) in items.prefix(options.values.layers.count).enumerated() {
            let layer = SectionReader(item)
            layer.bool("enabled", &options.values.layers[index].enabled)
            layer.double("denoise", &options.values.layers[index].denoise)
            layer.double("sharpen", &options.values.layers[index].sharpen)
            layer.int("level", &options.values.layers[index].level)
        }
    }
    read.bool("functionsOpen", &options.functionsOpen)
    if case .array(let items)? = read.values["functions"] {
        options.functions = items.compactMap { if case .string(let value) = $0 { value } else { nil } }
    }
    read.int("contrast", &tone.contrast)
    read.int("brightness", &tone.brightness)
    read.bool("holdContrast", &tone.holdContrast)

    let card = SectionReader(read.values["tone"])
    let shifts = SectionReader(card.values["shifts"])
    func channel(_ key: String, _ target: inout ChannelShift) {
        let reader = SectionReader(shifts.values[key])
        var x = Double(target.x)
        var y = Double(target.y)
        reader.double("x", &x)
        reader.double("y", &y)
        target = ChannelShift(x: Float(x), y: Float(y))
    }
    channel("red", &tone.shifts.red)
    channel("green", &tone.shifts.green)
    channel("blue", &tone.shifts.blue)
    card.int("maxShift", &tone.maxShift)
    card.int("red", &tone.red)
    card.int("green", &tone.green)
    card.int("blue", &tone.blue)
    card.int("black", &tone.black)
    card.int("white", &tone.white)
    card.double("clip", &tone.clip)
    if case .string(let value)? = card.values["histogramView"] {
        if value == "input" { tone.histogramView = .input } else if value == "result" { tone.histogramView = .result }
    }
    card.double("gamma", &tone.gamma)
}

/// The section `colour`: the check marks Colour and LRGB, which sit in the header of every tab in the original.
nonisolated func colourSection(_ options: WaveletOptions) -> JSON {
    .object(["colour": .bool(options.colour), "lrgb": .bool(options.lrgb)])
}

nonisolated func applyColourSection(_ section: JSON?, to options: inout WaveletOptions) {
    let read = SectionReader(section)
    read.bool("colour", &options.colour)
    read.bool("lrgb", &options.lrgb)
}
