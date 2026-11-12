import Foundation
import SidereaPlanetsKit

// The pure parts of the cards Denoise/Deringing and Masking: their values with the defaults and limits, what is sent to the core, the
// check for "changes nothing", and the part of the project file they are saved in. The calls are in AppModel+Wavelets.swift; the
// algorithms are in the Rust core (crates/core/src/refine.rs, docs/wavelets.md).

nonisolated enum RefineLimits {
    /// Denoise strength and Deringing amount, in percent.
    static let percent = 0...100
    /// How far from an edge Deringing looks for the dark ring, in pixels.
    static let radius = 1...10
    /// Brightness of the stack on a 0...255 scale.
    static let threshold = 0...255
    /// Width of the soft edge of the mask, in pixels.
    static let feather = 0...50
}

/// The values of the two cards. All of them go to the core except `showMask`, which only decides what the image area shows.
nonisolated struct RefineValues: Equatable, Sendable {
    var denoise = 0
    var deringing = 0
    var radius = 4
    var maskOn = false
    var threshold = 20
    var feather = 4
    var invert = false
    /// Show mask: the image area shows where the sharpening applies, tinted blue. Not saved and not part of the result.
    var showMask = false

    init() {}

    init(core settings: RefineSettings) {
        denoise = Self.clamp(Int(settings.denoise.rounded()), RefineLimits.percent)
        deringing = Self.clamp(Int(settings.deringing.rounded()), RefineLimits.percent)
        radius = Self.clamp(Int(settings.deringingRadius), RefineLimits.radius)
        maskOn = settings.mask.enabled
        threshold = Self.clamp(Int(settings.mask.threshold.rounded()), RefineLimits.threshold)
        feather = Self.clamp(Int(settings.mask.feather.rounded()), RefineLimits.feather)
        invert = settings.mask.invert
    }

    /// The values *Reset* restores. They come from the core, so Windows and Mac cannot drift apart.
    static var defaults: RefineValues { RefineValues(core: refineDefaultSettings()) }

    static func clamp(_ value: Int, _ range: ClosedRange<Int>) -> Int { min(range.upperBound, max(range.lowerBound, value)) }

    var core: RefineSettings {
        RefineSettings(
            denoise: Float(denoise), deringing: Float(deringing), deringingRadius: UInt32(radius),
            mask: MaskSettings(enabled: maskOn, threshold: Float(threshold), feather: Float(feather), invert: invert)
        )
    }

    /// The card Denoise/Deringing back to its defaults.
    mutating func resetDenoise() {
        let d = Self.defaults
        denoise = d.denoise
        deringing = d.deringing
        radius = d.radius
    }

    /// The card Masking back to its defaults: off.
    mutating func resetMask() {
        let d = Self.defaults
        maskOn = d.maskOn
        threshold = d.threshold
        feather = d.feather
        invert = d.invert
        showMask = false
    }
}

/// True when the values leave the image as it is (the core's `RefineSettings::is_neutral`).
nonisolated func isNeutral(_ refine: RefineSettings) -> Bool {
    refine.denoise <= 0 && refine.deringing <= 0 && !refine.mask.enabled
}

// MARK: Project file

/// The cards as the section `refine` of the project file, next to `wavelet` and `colour`, whose layout stays that of the Windows app. The
/// Windows app does not know the section and ignores it.
nonisolated func refineSection(_ values: RefineValues) -> JSON {
    .object([
        "denoise": .number(Double(values.denoise)),
        "deringing": .number(Double(values.deringing)),
        "radius": .number(Double(values.radius)),
        "maskOn": .bool(values.maskOn),
        "threshold": .number(Double(values.threshold)),
        "feather": .number(Double(values.feather)),
        "invert": .bool(values.invert),
    ])
}

/// Reads the object back; a missing or wrong key keeps the value, a number outside the limits is cut to them.
nonisolated func applyRefineSection(_ section: JSON?, to values: inout RefineValues) {
    let read = SectionReader(section)
    read.int("denoise", &values.denoise)
    read.int("deringing", &values.deringing)
    read.int("radius", &values.radius)
    read.bool("maskOn", &values.maskOn)
    read.int("threshold", &values.threshold)
    read.int("feather", &values.feather)
    read.bool("invert", &values.invert)
    values.denoise = RefineValues.clamp(values.denoise, RefineLimits.percent)
    values.deringing = RefineValues.clamp(values.deringing, RefineLimits.percent)
    values.radius = RefineValues.clamp(values.radius, RefineLimits.radius)
    values.threshold = RefineValues.clamp(values.threshold, RefineLimits.threshold)
    values.feather = RefineValues.clamp(values.feather, RefineLimits.feather)
}
