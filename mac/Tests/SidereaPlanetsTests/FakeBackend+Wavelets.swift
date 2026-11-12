import Foundation
import SidereaPlanetsKit

@testable import SidereaPlanets

/// What the Wavelets calls of the fake were asked, and what it answers. Calls arrive from the main actor and from the actor of the fake,
/// so the state lives behind a lock.
final class WaveletLog: @unchecked Sendable {
    struct Preview {
        var settings: WaveletSettings?
        var area: Region
        var mode: ColourMode
    }

    private let lock = NSLock()
    private var previewCalls: [Preview] = []
    private var doAllCalls: [(settings: WaveletSettings, mode: ColourMode)] = []
    private var sources: [RgbaImage] = []
    private var applied: [ColorSettings] = []
    private var histograms: [(settings: ColorSettings, after: Bool)] = []
    private var holdsDoAllFlag = false
    private var releasedDoAll = false
    private var previewDelay = Duration.zero

    var previews: [Preview] { lock.withLock { previewCalls } }
    var doAlls: [(settings: WaveletSettings, mode: ColourMode)] { lock.withLock { doAllCalls } }
    var colourSources: [RgbaImage] { lock.withLock { sources } }
    var appliedColours: [ColorSettings] { lock.withLock { applied } }
    var histogramRequests: [(settings: ColorSettings, after: Bool)] { lock.withLock { histograms } }
    var holdsDoAll: Bool {
        get { lock.withLock { holdsDoAllFlag } }
        set { lock.withLock { holdsDoAllFlag = newValue; releasedDoAll = false } }
    }
    var isDoAllReleased: Bool { lock.withLock { releasedDoAll } }
    var delayOfPreview: Duration {
        get { lock.withLock { previewDelay } }
        set { lock.withLock { previewDelay = newValue } }
    }

    func record(preview: Preview) { lock.withLock { previewCalls.append(preview) } }
    func record(doAll: (settings: WaveletSettings, mode: ColourMode)) { lock.withLock { doAllCalls.append(doAll) } }
    func record(source: RgbaImage) { lock.withLock { sources.append(source) } }
    func record(applied settings: ColorSettings) { lock.withLock { applied.append(settings) } }
    func record(histogram: (settings: ColorSettings, after: Bool)) { lock.withLock { histograms.append(histogram) } }
    func releaseDoAll() { lock.withLock { releasedDoAll = true } }
}

/// A flat picture of `value` in a 6 x 4 image.
func flatImage(_ value: UInt8, width: UInt32 = 6, height: UInt32 = 4) -> RgbaImage {
    var rgba = Data(count: Int(width * height) * 4)
    for i in stride(from: 0, to: rgba.count, by: 4) {
        rgba[i] = value
        rgba[i + 1] = value
        rgba[i + 2] = value
        rgba[i + 3] = 255
    }
    return RgbaImage(width: width, height: height, rgba: rgba)
}

extension FakeBackend {
    /// The pixel values of the fake: the stack is 100 everywhere, a preview 200, Do All 50, the cards 7.
    static let stackValue: UInt8 = 100
    static let previewValue: UInt8 = 200
    static let doAllValue: UInt8 = 50
    static let colourValue: UInt8 = 7

    func stackedImage() async throws -> RgbaImage {
        flatImage(Self.stackValue)
    }

    func waveletPreview(_ settings: WaveletSettings?, area: Region, mode: ColourMode) async throws -> WaveletPreview {
        wavelets.record(preview: WaveletLog.Preview(settings: settings, area: area, mode: mode))
        let delay = wavelets.delayOfPreview
        if delay > .zero { try await Task.sleep(for: delay) }
        let pixels = flatImage(Self.previewValue, width: area.width, height: area.height)
        return WaveletPreview(region: area, rgba: pixels.rgba)
    }

    func waveletDoAll(_ settings: WaveletSettings, mode: ColourMode, progress: ProgressHandler?) async throws -> RgbaImage {
        wavelets.record(doAll: (settings, mode))
        for done in 1...3 {
            progress?(OperationProgress(operation: .wavelet, done: UInt32(done), total: 6, elapsed: .milliseconds(Int64(done) * 10)))
        }
        if wavelets.holdsDoAll {
            while !wavelets.isDoAllReleased {
                if log.cancelledOperations.contains(.wavelet) { throw PlanetsError.Cancelled(message: "cancelled") }
                try await Task.sleep(for: .milliseconds(5))
            }
        }
        progress?(OperationProgress(operation: .wavelet, done: 6, total: 6, elapsed: .milliseconds(100)))
        return flatImage(Self.doAllValue)
    }

    func setColorSource(_ image: RgbaImage) async throws -> SourceInfo {
        wavelets.record(source: image)
        return SourceInfo(width: image.width, height: image.height, channels: 3)
    }

    func estimateRgbAlign(maxShift: UInt32) async throws -> ChannelShifts {
        ChannelShifts(red: ChannelShift(x: 1.5, y: -0.5), green: ChannelShift(x: 0, y: 0), blue: ChannelShift(x: -2, y: 1))
    }

    func applyColors(_ settings: ColorSettings) async throws -> RgbaImage {
        wavelets.record(applied: settings)
        return flatImage(Self.colourValue)
    }

    func colorHistogram(_ settings: ColorSettings, after: Bool) async throws -> HistogramData {
        wavelets.record(histogram: (settings, after))
        return HistogramData(channels: [[1, 2, 3], [3, 2, 1], [0, 5, 0]], luma: [1, 4, 1], bins: 3)
    }

    func autoHistogramPoints(_ settings: ColorSettings, clip: Float) async throws -> HistogramPoints {
        HistogramPoints(black: 12.4, white: 230.6)
    }
}
