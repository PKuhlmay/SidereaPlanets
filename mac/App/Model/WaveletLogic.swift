import CoreGraphics
import Foundation
import SidereaPlanetsKit

// The pure parts of the step Wavelets: the values of the panel with its defaults, what is sent to the core, linked
// sliders, the Processing Area, and the values and helpers of the Functions cards. Same rules as app/src/logic/wavelet.ts,
// logic/colour.ts and the tone part of logic/settings.ts; the calls themselves are in AppModel+Wavelets.swift.

// MARK: Layers

nonisolated enum WaveletLimits {
    static let layerCount = 6
    /// The slider position is ten times the factor of the layer (10 = 1.0): -5.0 ... 100.0.
    static let level = -50...1000
    /// What the core accepts for Denoise and Sharpen (`WaveletSettings::clamped`).
    static let amount = 0.0...100.0
}

/// One layer of the panel: check mark, Denoise, Sharpen and the slider.
nonisolated struct WaveletLayer: Equatable, Sendable {
    var enabled = true
    var denoise = 0.0
    var sharpen = 0.1
    /// Slider position, -50 ... 1000; the Preview button shows it divided by 10.
    var level = 10
}

/// The values that decide the result of the wavelets: what the core is sent.
nonisolated struct WaveletValues: Equatable, Sendable {
    var scheme = WaveletScheme.linear
    var filter = WaveletFilter.gaussian
    var initialLayer = 1
    var stepIncrement = 0
    var layers = Array(repeating: WaveletLayer(), count: WaveletLimits.layerCount)

    /// The values *Reset Wavelets* and a fresh start use. They come from the core, so Windows and Mac cannot drift apart
    /// (Linear, Gaussian, Initial 1, Step 0, six layers at 1.0, Denoise 0.00, Sharpen 0.100).
    static var defaults: WaveletValues { WaveletValues(core: waveletDefaultSettings()) }

    init() {}

    init(core settings: WaveletSettings) {
        scheme = settings.scheme
        filter = settings.filter
        initialLayer = Int(settings.initial)
        stepIncrement = Int(settings.step)
        layers = (0..<WaveletLimits.layerCount).map { index in
            guard settings.layers.indices.contains(index) else { return WaveletLayer() }
            let layer = settings.layers[index]
            return WaveletLayer(
                enabled: layer.enabled, denoise: Double(layer.denoise), sharpen: Double(layer.sharpen),
                level: Int((Double(layer.gain) * 10).rounded())
            )
        }
    }

    /// The values as the core takes them (`toBackend` of wavelet.ts).
    var core: WaveletSettings {
        WaveletSettings(
            scheme: scheme, filter: filter, initial: wholeNumber(initialLayer, minimum: 1), step: wholeNumber(stepIncrement, minimum: 0),
            layers: layers.map {
                LayerSettings(enabled: $0.enabled, gain: Float($0.level) / 10, denoise: Float($0.denoise), sharpen: Float($0.sharpen))
            }
        )
    }
}

/// Everything the Wavelets panel holds besides the values of the layers (`settings.wavelet`, `settings.colour` and `settings.lrgb` of
/// the Windows app).
nonisolated struct WaveletOptions: Equatable, Sendable {
    var values = WaveletValues.defaults
    /// Colour and LRGB belong together: they decide how the colour is treated while sharpening.
    var colour = true
    var lrgb = false
    /// Automatic: once the values stop changing, Do All runs by itself.
    var automatic = false
    /// Hold Wavelet Setting: a new stack keeps the values of the layers.
    var hold = false
    var linked = false
    // The Show menu of the Wavelet tab
    var showFullImage = false
    var showAlignpoints = false
    var showProcessingArea = false
    /// Settings > Processing Area: the side of the square the preview calculates.
    var processingAreaSide = 512
    /// The right panel (Functions »).
    var functionsOpen = true
    /// The pressed Functions buttons in the order they were pressed; each opens its own card.
    var functions: [String] = []

    var mode: ColourMode { colourMode(colour: colour, lrgb: lrgb) }
}

/// Colour and LRGB decide how the stack is sharpened: per channel, on the luminance only, or as grayscale. LRGB only counts with Colour.
nonisolated func colourMode(colour: Bool, lrgb: Bool) -> ColourMode {
    if !colour { return .gray }
    return lrgb ? .lrgb : .colour
}

/// Use Linked Wavelets: moving one slider moves all the others by the same amount. Which layers are linked is unobserved
/// (docs/offene-fragen.md); this is the simplest reading. Returns the new slider positions.
nonisolated func linkedLevels(_ levels: [Int], moving index: Int, to level: Int, linked: Bool) -> [Int] {
    func clamp(_ value: Int) -> Int { min(WaveletLimits.level.upperBound, max(WaveletLimits.level.lowerBound, value)) }
    let next = clamp(level)
    guard levels.indices.contains(index) else { return levels }
    if !linked { return levels.enumerated().map { $0.offset == index ? next : $0.element } }
    let delta = next - levels[index]
    return levels.enumerated().map { $0.offset == index ? next : clamp($0.element + delta) }
}

/// The Preview button of a layer: the slider position divided by ten, one decimal.
nonisolated func formatPreview(_ level: Int) -> String {
    String(format: "%.1f", Double(level) / 10)
}

/// The preview area: the square in the middle of the image (smaller if the image is), `side` from Settings > Preview area.
nonisolated func processingArea(imageWidth: Int, imageHeight: Int, side: Int) -> Region {
    let width = min(side, imageWidth)
    let height = min(side, imageHeight)
    return Region(
        x: UInt32(max((imageWidth - width) / 2, 0)), y: UInt32(max((imageHeight - height) / 2, 0)),
        width: UInt32(max(width, 0)), height: UInt32(max(height, 0))
    )
}

/// What a Do All result depends on: the stack, the values of the layers and the colour mode. The Processing Area only decides what
/// the preview shows, and the cards are applied when the image is saved, so neither belongs here.
nonisolated struct DoAllKey: Equatable, Sendable {
    var stack: Int
    var settings: WaveletSettings
    var mode: ColourMode
}

/// Copies the rows of a preview into the RGBA pixels of the whole image.
nonisolated func writePreview(_ preview: WaveletPreview, into pixels: inout Data, imageWidth: Int) {
    let region = preview.region
    let rowBytes = Int(region.width) * 4
    preview.rgba.withUnsafeBytes { source in
        pixels.withUnsafeMutableBytes { target in
            for row in 0..<Int(region.height) {
                let to = ((Int(region.y) + row) * imageWidth + Int(region.x)) * 4
                guard to + rowBytes <= target.count, (row + 1) * rowBytes <= source.count else { return }
                target.baseAddress!.advanced(by: to).copyMemory(from: source.baseAddress!.advanced(by: row * rowBytes), byteCount: rowBytes)
            }
        }
    }
}

// MARK: Functions cards and Contrast/Brightness

/// The buttons of the Adjustments panel (three columns, five rows); only the four with a card work.
nonisolated let functionNames = [
    "Histogram", "Gamma", "Color Mixing",
    "View Zoomed", "View Compare", "View Stack Size",
    "Flip and Rotate", "RGB Align", "RGB Balance",
    "Resize Image", "Denoise/Deringing", "Wavelet Filter",
    "Masking", "Show Line Graph", "Cropping Area",
]

/// The cards that exist; the other Functions buttons only report that they are not available yet.
nonisolated let toneCards = ["Histogram", "Gamma", "RGB Align", "RGB Balance"]

nonisolated func isToneCard(_ name: String) -> Bool { toneCards.contains(name) }

/// The tooltip of an Adjustments button: what its card does, or that the button is locked.
nonisolated func functionHelp(_ name: String) -> String {
    switch name {
    case "Histogram": "Black point, white point and brightness of the image."
    case "Gamma": "Brightens or darkens the midtones."
    case "RGB Align": "Shifts red, green and blue so that they lie on top of each other."
    case "RGB Balance": "Changes the share of red, green and blue."
    default: "Not available yet"
    }
}

nonisolated enum ToneLimits {
    static let shift = 20.0
    static let maxShift = 1...20
    static let balance = 0...200
    static let gamma = 0.1...5.0
    static let level = 0...255
    static let clip = 0.0...49.0
    static let contrast = 1...1000
    static let brightness = -512...512
}

/// Contrast starts at 250 on the slider while the field shows 100; the ratio 2.5 follows from that pair of values only
/// (docs/offene-fragen.md, question 23).
nonisolated let contrastRatio = 2.5

nonisolated func contrastField(_ slider: Int) -> Int { Int((Double(slider) / contrastRatio).rounded()) }

nonisolated func contrastSlider(_ field: Int) -> Int {
    min(ToneLimits.contrast.upperBound, max(ToneLimits.contrast.lowerBound, Int((Double(field) * contrastRatio).rounded())))
}

nonisolated enum HistogramView: Equatable, Sendable {
    case input, result
}

/// The values of the cards behind the Functions buttons and of Contrast/Brightness.
nonisolated struct ToneValues: Equatable, Sendable {
    /// RGB Align: shift of every channel in pixels.
    var shifts = ToneValues.noShifts
    /// RGB Align, Auto: how far the search goes, in pixels.
    var maxShift = 5
    /// RGB Balance in percent, 100 = unchanged.
    var red = 100
    var green = 100
    var blue = 100
    /// Histogram: black and white point on a 0...255 scale, the share of pixels Auto cuts off at each end, and the graph shown.
    var black = 0
    var white = 255
    var clip = 0.1
    var histogramView = HistogramView.input
    /// 1.0 = unchanged.
    var gamma = 1.0
    /// The slider positions; the Contrast field shows `contrastField(contrast)`.
    var contrast = 250
    var brightness = 0
    var holdContrast = false

    static let noShifts = ChannelShifts(
        red: ChannelShift(x: 0, y: 0), green: ChannelShift(x: 0, y: 0), blue: ChannelShift(x: 0, y: 0)
    )

    /// The values as the core takes them: all cards plus the sliders of Contrast/Brightness.
    var core: ColorSettings {
        ColorSettings(
            shifts: shifts, red: Float(red), green: Float(green), blue: Float(blue), black: Float(black), white: Float(white),
            gamma: Float(gamma), contrast: Float(contrastField(contrast)), brightness: Float(brightness)
        )
    }
}

/// The histogram a request would return. The one before the stretch does not depend on the stretch, gamma, contrast and brightness,
/// so moving those does not need a new one.
nonisolated struct HistogramKey: Equatable, Sendable {
    var colours: ColorSettings?
    var shifts: ChannelShifts
    var red: Float
    var green: Float
    var blue: Float
    var base: Int
    var after: Bool

    init(_ v: ColorSettings, after: Bool, base: Int) {
        self.after = after
        self.base = base
        shifts = v.shifts
        red = v.red
        green = v.green
        blue = v.blue
        colours = after ? v : nil
    }
}

/// Height of a bar on a logarithmic scale, 0...1: a few bright pixels stay visible next to a black sky.
nonisolated func logHeight(_ count: Double, max maximum: Double) -> Double {
    maximum <= 0 ? 0 : log1p(count) / log1p(maximum)
}

nonisolated func histogramMax(_ data: HistogramData) -> Double {
    Double(max(data.luma.max() ?? 0, data.channels.flatMap { $0 }.max() ?? 0))
}

/// The outline of one series in a box of `width` x `height`; the caller closes it down to the axis for the filled area.
nonisolated func histogramPoints(_ counts: [UInt32], max maximum: Double, width: Double, height: Double) -> [CGPoint] {
    counts.enumerated().map { index, count in
        CGPoint(
            x: counts.count == 1 ? 0 : Double(index) / Double(counts.count - 1) * width,
            y: height - logHeight(Double(count), max: maximum) * height
        )
    }
}

/// Position on the graph (0...width) to a level on the 0...255 scale, and back.
nonisolated func xToLevel(_ x: Double, width: Double) -> Int { min(255, max(0, Int((x / width * 255).rounded()))) }
nonisolated func levelToX(_ level: Int, width: Double) -> Double { Double(level) / 255 * width }

/// The black marker stays at least one level below white, the white marker one above black.
nonisolated func moveBlack(_ level: Int, white: Int) -> Int { min(max(0, level), white - 1) }
nonisolated func moveWhite(_ level: Int, black: Int) -> Int { max(min(255, level), black + 1) }

/// Which image the Wavelets and Save steps show: the current one, the copied one or the difference between them.
nonisolated enum ToneView: Equatable, Sendable {
    case current, clipboard, difference
}

/// How much the difference is amplified, so that small changes show. Our choice.
nonisolated let differenceGain = 4

/// |a - b| per colour channel times `differenceGain`, opaque; `nil` if the sizes differ.
nonisolated func differencePixels(_ a: RgbaImage, _ b: RgbaImage) -> RgbaImage? {
    guard a.width == b.width, a.height == b.height, a.rgba.count == b.rgba.count else { return nil }
    var out = Data(count: a.rgba.count)
    a.rgba.withUnsafeBytes { left in
        b.rgba.withUnsafeBytes { right in
            out.withUnsafeMutableBytes { target in
                let (l, r, t) = (left.bindMemory(to: UInt8.self), right.bindMemory(to: UInt8.self), target.bindMemory(to: UInt8.self))
                for i in stride(from: 0, to: t.count, by: 4) {
                    for c in 0..<3 {
                        let d = Int(l[i + c]) - Int(r[i + c])
                        t[i + c] = UInt8(min(255, abs(d) * differenceGain))
                    }
                    t[i + 3] = 255
                }
            }
        }
    }
    return RgbaImage(width: a.width, height: a.height, rgba: out)
}
