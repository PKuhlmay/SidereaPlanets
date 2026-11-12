import CoreGraphics
import Foundation
import SidereaPlanetsKit

// The pure parts of the steps Alignpoints, Align and Limit: the values of the panels with the defaults of the reference program, what is
// sent to the core, and how its answers become what the window shows. Same rules as app/src/logic/align.ts, qualityChart.ts and
// chart.ts; the calls themselves are in AppModel+Align.swift.

/// *Limit Setup*: which frames stay.
nonisolated enum LimitMode: String, CaseIterable, Identifiable, Sendable {
    case lowestQuality, bestPercent, framesPerPoint, bestCount

    var id: String { rawValue }

    var label: String {
        switch self {
        case .lowestQuality: "Lowest Quality (%)"
        case .bestPercent: "Best Frames (%)"
        case .framesPerPoint: "Frames/Apoint"
        case .bestCount: "Best Frames"
        }
    }

    /// The two modes that count frames use the number field, the two percent modes the percent field.
    var countsFrames: Bool { self == .framesPerPoint || self == .bestCount }
}

/// The values of the panels Alignpoints, Align and Limit (`settings.align` of the Windows app), with the defaults observed in
/// the reference program (docs/ui.md). Typed values outside the usual ranges are accepted; they are only held to what the core can take
/// (`wholeNumber`) when they are sent.
nonisolated struct AlignOptions: Equatable, Sendable {
    // Alignpoints
    var minDistance = 30
    var edgeDistance = 20
    var intensity = Intensity.default
    var lo = 30
    var hi = 230
    // Align
    var boxSize = 30
    var maxMovement = 5
    var showAlignment = false
    // Limit
    var limitMode = LimitMode.lowestQuality
    var limitPercent = 80
    var limitCount = 200
}

/// The check marks of the Show menu in the steps Alignpoints, Align and Limit (the ones of the Align tab of the original).
nonisolated struct ShowOptions: Equatable, Sendable {
    var fullImage = false
    var roi = false
    var alignpoints = true
    var aligndata = false
    var framelist = false
    var registrationgraph = false
    var prefilter = false
}

/// One line of the Framelist: whether the frame stays and what Align rated it.
nonisolated struct FrameEntry: Equatable, Identifiable, Sendable {
    let index: Int
    var checked = true
    /// In percent of the best frame; `nil` before Align.
    var quality: Double?
    var difference: Double?

    var id: Int { index }
    /// "Frame00010": the name the Framelist shows.
    var name: String { String(format: "Frame%05d", index + 1) }
}

/// What the Align tab has found: the points, the rating of the frames and the region.
nonisolated struct AlignData: Equatable, Sendable {
    var points: [Point] = []
    /// How many points satisfy the spacing rules before the threshold: the upper end of the Threshold slider (0 = none set).
    var total = 0
    var threshold = 0
    /// The frame the points were set on; Align and Stack register the other frames against it.
    var referenceFrame = 0
    var frames: [FrameEntry] = []
    /// The region of interest of Set Alignpoints and Align, in image pixels; `nil` is the whole frame.
    var roi: RoiSettings?

    init(frameCount: Int = 0) {
        frames = (0..<frameCount).map { FrameEntry(index: $0) }
    }

    var isRated: Bool { frames.contains { $0.quality != nil } }

    mutating func show(_ answer: Alignpoints) {
        points = answer.points
        total = Int(answer.total)
        threshold = Int(answer.threshold)
    }

    /// A new Set Alignpoints or Align replaces the alignment: the rating and the choice of frames go.
    mutating func discardRating() {
        for i in frames.indices {
            frames[i].checked = true
            frames[i].quality = nil
            frames[i].difference = nil
        }
    }

    /// The Framelist: in frame order, after Align the best frame first.
    var listed: [FrameEntry] {
        isRated ? frames.sorted { ($0.quality ?? -1, -$0.index) > ($1.quality ?? -1, -$1.index) } : frames
    }

    /// The best quality in percent, `nil` before Align.
    var bestQuality: Double? { frames.compactMap(\.quality).max() }

    var keptCount: Int { frames.filter(\.checked).count }
}

// MARK: What is sent to the core

/// A typed value as a whole number of at least `minimum`, for the calls that take an unsigned number.
nonisolated func wholeNumber(_ value: Int, minimum: Int) -> UInt32 {
    UInt32(clamping: max(minimum, value))
}

nonisolated func pointSettings(_ options: AlignOptions, roi: RoiSettings?) -> PointSettings {
    PointSettings(
        minDistance: wholeNumber(options.minDistance, minimum: 0),
        edgeDistance: wholeNumber(options.edgeDistance, minimum: 0),
        maxMovement: wholeNumber(options.maxMovement, minimum: 0),
        roi: roi,
        intensity: options.intensity,
        lo: Float(options.lo),
        hi: Float(options.hi),
        boxSize: wholeNumber(options.boxSize, minimum: 1)
    )
}

nonisolated func alignSettings(_ options: AlignOptions, roi: RoiSettings?) -> AlignSettings {
    AlignSettings(
        boxSize: wholeNumber(options.boxSize, minimum: 1),
        maxMovement: wholeNumber(options.maxMovement, minimum: 0),
        roi: roi
    )
}

/// Lowest Quality (%) and Best Frames (%) take the percent field, the two count modes the number field.
nonisolated func limitSettings(_ options: AlignOptions) -> LimitSettings {
    let percent = Float(options.limitPercent)
    let count = UInt64(wholeNumber(options.limitCount, minimum: 1))
    switch options.limitMode {
    case .lowestQuality: return .lowestQuality(value: percent)
    case .bestPercent: return .bestPercent(value: percent)
    case .framesPerPoint: return .framesPerPoint(value: count)
    case .bestCount: return .bestCount(value: count)
    }
}

// MARK: What the core answers

/// Quality in percent of the best frame, as the Framelist shows it.
nonisolated func qualityPercent(_ qualities: [Float]) -> [Double] {
    let best = Double(qualities.max() ?? 0)
    return qualities.map { best > 0 ? Double($0) / best * 100 : 0 }
}

/// The rectangle spanned by two corners (in image pixels), kept inside the image and with whole pixels.
nonisolated func rectBetween(_ a: CGPoint, _ b: CGPoint, image: CGSize) -> RoiSettings {
    func clamp(_ value: CGFloat, _ maximum: CGFloat) -> UInt32 {
        UInt32(min(maximum, max(0, value.rounded())))
    }
    let x0 = clamp(min(a.x, b.x), image.width)
    let y0 = clamp(min(a.y, b.y), image.height)
    let x1 = clamp(max(a.x, b.x), image.width)
    let y1 = clamp(max(a.y, b.y), image.height)
    return RoiSettings(x: x0, y: y0, width: x1 - x0, height: y1 - y0)
}

/// A rectangle dragged by accident (a click) is not a region.
nonisolated let minimumRegionSide: UInt32 = 8

nonisolated func isRegion(_ rect: RoiSettings) -> Bool {
    rect.width >= minimumRegionSide && rect.height >= minimumRegionSide
}

/// Whether a region leaves room for any point: it must be wider and higher than twice *Min distance from edge*.
nonisolated func roomForPoints(_ rect: RoiSettings, edgeDistance: Int) -> Bool {
    Int(rect.width) > 2 * edgeDistance && Int(rect.height) > 2 * edgeDistance
}

/// The status bar after Limit: how many frames stay and, as "Low quality", the quality of the weakest of them in percent of the
/// best frame. That meaning is a conjecture (docs/workflow.md): with all ten frames of the reference case the original reads 100,00 %.
nonisolated func limitMessages(kept: Int, keptQualities: [Double]) -> (action: String, result: String) {
    let action = "Processing limited to \(kept) frames"
    guard let weakest = keptQualities.min() else { return (action, "Stacksize: \(kept)") }
    return (action, "Stacksize: \(kept)  Low quality: \(String(format: "%.2f", weakest))%")
}

/// "00:00:03" left of a run that is `progress` (0...1) done after `elapsed` seconds; unknown at the very start.
nonisolated func timeLeft(elapsed: Double, progress: Double) -> Double {
    progress <= 0 ? 0 : elapsed * (1 - progress) / progress
}

// MARK: Time of a run

/// The time a run has used, without the time it was paused.
nonisolated struct JobClock {
    private let start: ContinuousClock.Instant
    private var pausedAt: ContinuousClock.Instant?
    private var pausedTotal = Duration.zero

    init(now: ContinuousClock.Instant = .now) {
        start = now
    }

    mutating func pause(now: ContinuousClock.Instant = .now) {
        if pausedAt == nil { pausedAt = now }
    }

    mutating func resume(now: ContinuousClock.Instant = .now) {
        if let pausedAt { pausedTotal += now - pausedAt }
        pausedAt = nil
    }

    /// In seconds.
    func used(now: ContinuousClock.Instant = .now) -> Double {
        let paused = pausedTotal + (pausedAt.map { now - $0 } ?? .zero)
        let used = now - start - paused
        return Double(used.components.seconds) + Double(used.components.attoseconds) / 1e18
    }
}

// MARK: Quality chart

/// One bar of the quality chart.
nonisolated struct QualityBar: Equatable, Sendable {
    /// Quality as a share of the best frame, 0...1.
    var height: Double
    var kept: Bool
}

/// Where the limit sits in the chart.
nonisolated enum LimitLine: Equatable, Sendable {
    /// Lowest Quality (%): a horizontal line at this share of the best quality, 0...1.
    case level(Double)
    /// The other modes: a vertical line after the last kept frame, as a share of the width, 0...1.
    case after(Double)
}

nonisolated struct QualityChart: Equatable, Sendable {
    var bars: [QualityBar] = []
    var frames = 0
    var kept = 0
    var line: LimitLine?
    /// The limit value next to the line.
    var value = ""
    /// Text alternative of the chart.
    var label = "No frame is rated yet."
}

/// How many of the frames (qualities best first) a limit keeps; at least one if there is any. Frames/Apoint depends on the
/// quality of every point in every frame, which only the core knows, so it has no count here. Mirrors `limit_frames` in
/// crates/core/src/align.rs, so that the chart follows the limit value before Limit runs.
nonisolated func keptCount(sorted: [Double], options: AlignOptions) -> Int? {
    let n = sorted.count
    if n == 0 { return 0 }
    let keep: Int
    switch options.limitMode {
    case .lowestQuality:
        let floor = sorted[0] * Double(options.limitPercent) / 100
        keep = sorted.filter { $0 >= floor }.count
    case .bestPercent:
        keep = Int((Double(n) * Double(options.limitPercent) / 100).rounded(.up))
    case .bestCount:
        keep = options.limitCount
    case .framesPerPoint:
        return nil
    }
    return min(n, max(1, keep))
}

/// The bars of the chart, best frame first. With more frames than `maxBars` one bar stands for several neighbouring frames: its
/// height is their mean quality and it counts as kept when at least half of them are kept.
nonisolated func qualityChart(frames: [FrameEntry], options: AlignOptions, maxBars: Int) -> QualityChart {
    let rated = frames
        .compactMap { entry in entry.quality.map { (quality: $0, checked: entry.checked) } }
        .sorted { $0.quality > $1.quality }
    let n = rated.count
    guard n > 0 else { return QualityChart() }

    let best = rated[0].quality
    let count = keptCount(sorted: rated.map(\.quality), options: options)
    // Frames/Apoint: the frames the last Limit kept.
    let kept = rated.enumerated().map { index, frame in count.map { index < $0 } ?? frame.checked }
    let keptTotal = kept.filter { $0 }.count
    func share(_ quality: Double) -> Double { best > 0 ? min(1, max(0, quality / best)) : 0 }

    let size = max(1, Int((Double(n) / Double(max(1, maxBars))).rounded(.up)))
    var bars: [QualityBar] = []
    for start in stride(from: 0, to: n, by: size) {
        let end = min(start + size, n)
        let mean = rated[start..<end].reduce(0) { $0 + $1.quality } / Double(end - start)
        let keptInGroup = kept[start..<end].filter { $0 }.count
        bars.append(QualityBar(height: share(mean), kept: keptInGroup * 2 >= end - start))
    }

    let lastKept = kept.lastIndex(of: true).map { $0 + 1 } ?? 0
    let line: LimitLine =
        options.limitMode == .lowestQuality
        ? .level(min(1, max(0, Double(options.limitPercent) / 100)))
        : .after(Double(lastKept) / Double(n))
    let value = options.limitMode.countsFrames ? String(options.limitCount) : "\(options.limitPercent) %"
    return QualityChart(
        bars: bars, frames: n, kept: keptTotal, line: line, value: value,
        label: "Quality of \(n) frames, best first. \(keptTotal) \(keptTotal == 1 ? "is" : "are") kept."
    )
}

/// The points of one series of the Registrationgraph, scaled to its own maximum; the first value sits left, the last right.
/// `y` counts from the top of `box`.
nonisolated func seriesPoints(_ values: [Double], in box: CGRect) -> [CGPoint] {
    let maximum = values.max() ?? 0
    guard !values.isEmpty, maximum > 0 else { return [] }
    let dx = values.count > 1 ? box.width / CGFloat(values.count - 1) : 0
    return values.enumerated().map { i, v in
        CGPoint(x: box.minX + CGFloat(i) * dx, y: box.minY + box.height * (1 - v / maximum))
    }
}
