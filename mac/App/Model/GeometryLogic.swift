import Foundation
import SidereaPlanetsKit

// The pure parts of the cards Flip and Rotate and Resize Image (MAC-017): their values with the defaults and limits, the sizes they
// result in, what is sent to the core, and the section `geometry` of the project file. The picture is made in the Rust core
// (crates/core/src/geometry.rs); the order there is refine, colors, flip and rotate, crop, resize. The model methods are in
// AppModel+Geometry.swift.

nonisolated enum OrientationLimits {
    /// The free angle in degrees, clockwise; the field and the slider move in tenths.
    static let angle = -180.0...180.0
    static let angleStep = 0.1
}

nonisolated enum ResizeLimits {
    /// The share of the size in percent.
    static let percent = 10...400
    /// A side in pixels (`MAX_SIDE` of the core).
    static let side = 1...16_384
}

/// The two ways to look up pixels between pixels when a picture is turned by a free angle.
nonisolated enum RotationMethod: String, CaseIterable, Sendable {
    case bilinear, bicubic

    var title: String { self == .bilinear ? "Bilinear" : "Bicubic" }
    var core: Resampling { self == .bilinear ? .bilinear : .bicubic }
}

/// The values of the card Flip and Rotate. The picture is mirrored first (in its own frame) and then turned clockwise by `angle`; the
/// buttons act on what is on screen, so a flip also mirrors the angle (see `flipHorizontally`).
nonisolated struct OrientationValues: Equatable, Sendable {
    var flipHorizontal = false
    var flipVertical = false
    /// Clockwise in degrees, from -180 to 180 in tenths. Multiples of 90 turn exactly, anything between is resampled.
    var angle = 0.0
    /// Cuts a freely turned picture to the largest part of the original shape without black corners.
    var cropToFit = false
    var method = RotationMethod.bicubic

    /// Whether the picture stays as it is.
    var isNeutral: Bool { !flipHorizontal && !flipVertical && abs(angle) < 0.001 }

    /// The values as the core takes them.
    var core: OrientationSettings {
        OrientationSettings(
            flipHorizontal: flipHorizontal, flipVertical: flipVertical, angle: Float(angle), cropToFit: cropToFit, method: method.core)
    }

    /// An angle in tenths of a degree, wrapped into -180 ... 180 (180 stays 180).
    static func wrapped(_ degrees: Double) -> Double {
        var value = (degrees * 10).rounded() / 10
        while value > 180 { value -= 360 }
        while value <= -180 { value += 360 }
        return value == 0 ? 0 : value
    }

    /// Flip Horizontal mirrors what is on screen: the mirror is done first in the frame of the picture, so the angle changes its sign.
    mutating func flipHorizontally() {
        flipHorizontal.toggle()
        angle = Self.wrapped(-angle)
    }

    /// Flip Vertical, likewise.
    mutating func flipVertically() {
        flipVertical.toggle()
        angle = Self.wrapped(-angle)
    }

    /// Turns the picture on screen by `degrees` clockwise (negative: counter-clockwise).
    mutating func rotate(by degrees: Double) {
        angle = Self.wrapped(angle + degrees)
    }

    mutating func setAngle(_ degrees: Double) {
        angle = Self.wrapped(min(max(degrees, OrientationLimits.angle.lowerBound), OrientationLimits.angle.upperBound))
    }

    /// The part of the values that decides the size and the place of the pixels (not the way they are looked up): when it changes, a crop
    /// rectangle set on the old picture no longer fits.
    var layout: [Double] { [flipHorizontal ? 1 : 0, flipVertical ? 1 : 0, angle, cropToFit ? 1 : 0] }

    /// "15.0° clockwise, flipped horizontally"; `nil` while nothing changes.
    var summary: String? {
        guard !isNeutral else { return nil }
        var parts: [String] = []
        if abs(angle) >= 0.001 {
            parts.append(String(format: "turned %.1f° %@", abs(angle), angle > 0 ? "clockwise" : "counterclockwise"))
        }
        if flipHorizontal { parts.append("flipped horizontally") }
        if flipVertical { parts.append("flipped vertically") }
        return parts.joined(separator: ", ")
    }
}

/// How a resize is given: as a share of the size, or as a size in pixels.
nonisolated enum ResizeMode: String, Sendable {
    case percent, pixels
}

/// The side of the picture the user typed last; with Lock aspect ratio the other side follows it.
nonisolated enum ResizeSide: String, Sendable {
    case width, height
}

/// The ways to scale. Lanczos3 is the sharpest for planets and the default; Nearest Neighbor copies pixels, for enlarging pixel by pixel.
nonisolated enum ResizeMethod: String, CaseIterable, Sendable {
    case lanczos3, bicubic, bilinear, nearest

    var title: String {
        switch self {
        case .lanczos3: "Lanczos3"
        case .bicubic: "Bicubic"
        case .bilinear: "Bilinear"
        case .nearest: "Nearest Neighbor"
        }
    }

    var core: Resampling {
        switch self {
        case .lanczos3: .lanczos3
        case .bicubic: .bicubic
        case .bilinear: .bilinear
        case .nearest: .nearest
        }
    }
}

/// The values of the card Resize Image. In pixels, `width` and `height` of 0 mean "not typed yet": the picture keeps its size.
nonisolated struct ResizeValues: Equatable, Sendable {
    var mode = ResizeMode.percent
    var percent = 100
    var width = 0
    var height = 0
    /// With the lock the side typed last (`driver`) decides and the other follows, so the shape is kept.
    var lockAspect = true
    var driver = ResizeSide.width
    var method = ResizeMethod.lanczos3

    /// The request for the core; with the lock only the side that was typed last is sent, so the other follows the shape.
    var core: ResizeSettings {
        switch mode {
        case .percent:
            return ResizeSettings(target: .percent(percent: Float(percent)), method: method.core)
        case .pixels:
            let wide = width > 0 ? UInt32(width) : nil
            let high = height > 0 ? UInt32(height) : nil
            guard lockAspect else { return ResizeSettings(target: .pixels(width: wide, height: high), method: method.core) }
            let target: ResizeTarget
            switch (driver, wide, high) {
            case (.width, let w?, _), (_, let w?, nil): target = .pixels(width: w, height: nil)
            case (.height, _, let h?), (_, nil, let h?): target = .pixels(width: nil, height: h)
            default: target = .pixels(width: nil, height: nil)
            }
            return ResizeSettings(target: target, method: method.core)
        }
    }

    /// The size the picture gets from a source of the given size.
    func size(source: (width: Int, height: Int)) -> (width: Int, height: Int) {
        let size = resizeSize(width: UInt32(source.width), height: UInt32(source.height), resize: core)
        return (Int(size.width), Int(size.height))
    }

    /// Whether the picture stays as it is.
    func isNeutral(source: (width: Int, height: Int)) -> Bool {
        let result = size(source: source)
        return result.width == source.width && result.height == source.height
    }

    /// A typed width: with the lock it decides the height, without it only the width changes.
    mutating func setWidth(_ value: Int, source: (width: Int, height: Int)) {
        pinSides(source: source)
        width = min(max(value, ResizeLimits.side.lowerBound), ResizeLimits.side.upperBound)
        driver = .width
    }

    mutating func setHeight(_ value: Int, source: (width: Int, height: Int)) {
        pinSides(source: source)
        height = min(max(value, ResizeLimits.side.lowerBound), ResizeLimits.side.upperBound)
        driver = .height
    }

    /// Without the lock both sides are fixed numbers: the side that was not typed yet takes the size it has now.
    private mutating func pinSides(source: (width: Int, height: Int)) {
        guard !lockAspect else { return }
        let now = size(source: source)
        if width == 0 { width = now.width }
        if height == 0 { height = now.height }
    }

    /// The pixel fields start from the size the picture has now, whatever mode it was before.
    mutating func switchToPixels(source: (width: Int, height: Int)) {
        let now = size(source: source)
        mode = .pixels
        width = now.width
        height = now.height
        driver = .width
    }

    mutating func switchToPercent() {
        mode = .percent
    }

    /// Turning the lock off fixes both sides at the size they show now.
    mutating func setLock(_ on: Bool, source: (width: Int, height: Int)) {
        if !on && mode == .pixels {
            let now = size(source: source)
            width = now.width
            height = now.height
        }
        lockAspect = on
    }
}

/// What the two cards hold.
nonisolated struct GeometryValues: Equatable, Sendable {
    var orientation = OrientationValues()
    var resize = ResizeValues()
}

/// "1200 × 900 px".
nonisolated func sizeText(_ width: Int, _ height: Int) -> String { "\(width) × \(height) px" }

// MARK: Project file

/// The section `geometry` of the project file, next to `wavelet` and `colour`. The Windows app does not know it and ignores it.
nonisolated func geometrySection(_ values: GeometryValues) -> JSON {
    let o = values.orientation
    let r = values.resize
    return .object([
        "flipHorizontal": .bool(o.flipHorizontal),
        "flipVertical": .bool(o.flipVertical),
        "angle": .number(o.angle),
        "cropToFit": .bool(o.cropToFit),
        "rotationMethod": .string(o.method.rawValue),
        "resize": .object([
            "mode": .string(r.mode.rawValue),
            "percent": .number(Double(r.percent)),
            "width": .number(Double(r.width)),
            "height": .number(Double(r.height)),
            "lockAspect": .bool(r.lockAspect),
            "driver": .string(r.driver.rawValue),
            "method": .string(r.method.rawValue),
        ]),
    ])
}

/// Reads the object back; a missing or wrong key keeps the value, a number outside the limits is cut to them.
nonisolated func applyGeometrySection(_ section: JSON?, to values: inout GeometryValues) {
    let read = SectionReader(section)
    read.bool("flipHorizontal", &values.orientation.flipHorizontal)
    read.bool("flipVertical", &values.orientation.flipVertical)
    var angle = values.orientation.angle
    read.double("angle", &angle)
    values.orientation.setAngle(angle)
    read.bool("cropToFit", &values.orientation.cropToFit)
    read.choice("rotationMethod", &values.orientation.method)

    let resize = SectionReader(read.values["resize"])
    resize.choice("mode", &values.resize.mode)
    resize.int("percent", &values.resize.percent)
    resize.int("width", &values.resize.width)
    resize.int("height", &values.resize.height)
    resize.bool("lockAspect", &values.resize.lockAspect)
    resize.choice("driver", &values.resize.driver)
    resize.choice("method", &values.resize.method)
    values.resize.percent = min(max(values.resize.percent, ResizeLimits.percent.lowerBound), ResizeLimits.percent.upperBound)
    values.resize.width = min(max(values.resize.width, 0), ResizeLimits.side.upperBound)
    values.resize.height = min(max(values.resize.height, 0), ResizeLimits.side.upperBound)
}
