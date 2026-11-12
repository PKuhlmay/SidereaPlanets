import Foundation
import SidereaPlanetsKit

// The pure parts of the Color Mixing card: the 3 x 3 matrix in percent, the presets, and what is sent to the core. Output channel
// `row` is the sum of the input channels times `matrix[row * 3 + column]`, in the order red, green, blue.

nonisolated enum MixLimits {
    /// One weight, in percent.
    static let weight = -200...200
}

/// The values of the Color Mixing card. The default is the identity, which leaves the image as it is.
nonisolated struct ColorMixValues: Equatable, Sendable {
    static let identityMatrix = [100, 0, 0, 0, 100, 0, 0, 0, 100]

    var matrix = ColorMixValues.identityMatrix
    var preserveBrightness = false

    /// Whether mixing would change nothing; Preserve brightness has nothing to keep then.
    var isIdentity: Bool { matrix == Self.identityMatrix }

    /// The values as the core takes them.
    var core: ColorMix {
        ColorMix(matrix: matrix.map { Float(clampedWeight($0)) }, preserveBrightness: preserveBrightness)
    }

    subscript(output: Int, input: Int) -> Int {
        get { matrix[output * 3 + input] }
        set { matrix[output * 3 + input] = clampedWeight(newValue) }
    }
}

nonisolated func clampedWeight(_ value: Int) -> Int {
    min(MixLimits.weight.upperBound, max(MixLimits.weight.lowerBound, value))
}

/// A ready-made matrix in the menu of the card. Preserve brightness stays as the user set it.
nonisolated struct MixPreset: Equatable, Sendable, Identifiable {
    let name: String
    let matrix: [Int]
    var id: String { name }
}

nonisolated let mixPresets = [
    MixPreset(name: "Neutral", matrix: ColorMixValues.identityMatrix),
    MixPreset(name: "Swap Red and Blue", matrix: [0, 0, 100, 0, 100, 0, 100, 0, 0]),
    // The weights of the luminance, so that all three channels show the brightness of the color image.
    MixPreset(name: "Luminance Gray", matrix: [30, 59, 11, 30, 59, 11, 30, 59, 11]),
    // For infrared-pass and other single-filter data: the red channel drives all three outputs.
    MixPreset(name: "Red to All Channels", matrix: [100, 0, 0, 100, 0, 0, 100, 0, 0]),
]

/// The values of a project file as numbers: nine finite weights, each held to the limits; `nil` for anything else.
nonisolated func mixMatrix(from numbers: [Double]) -> [Int]? {
    guard numbers.count == 9, numbers.allSatisfy(\.isFinite) else { return nil }
    return numbers.map { clampedWeight(Int($0.rounded())) }
}
