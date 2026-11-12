import Dispatch
import Foundation

// The types of crates/ffi (PointSettings, StackSettings, RgbaImage, ...) are the vocabulary of the app: re-export them.
@_exported import SidereaPlanetsBindings

/// How far a running operation is: `done` of `total` steps (frames, or layers for the wavelets).
public struct OperationProgress: Sendable, Equatable {
    public let operation: SessionOperation
    public let done: UInt32
    public let total: UInt32
    public let elapsed: Duration

    public init(operation: SessionOperation, done: UInt32, total: UInt32, elapsed: Duration) {
        self.operation = operation
        self.done = done
        self.total = total
        self.elapsed = elapsed
    }

    /// 0...1; 0 while `total` is not known yet.
    public var fraction: Double { total == 0 ? 0 : Double(done) / Double(total) }
}

/// Receives progress on the calculating thread, not on the main actor: hop to it before touching the UI.
public typealias ProgressHandler = @Sendable (OperationProgress) -> Void

extension PlanetsError {
    /// The run was stopped by `cancel`; nothing went wrong.
    public var isCancelled: Bool {
        if case .Cancelled = self { return true }
        return false
    }

    /// The text of the session error, the one the window shows (`localizedDescription` of the generated type does not
    /// carry it).
    public var message: String {
        switch self {
        case .NoRecording(let message), .NoAlignpoints(let message), .NotAligned(let message),
            .NotStacked(let message), .NoImageToSharpen(let message), .NoImageToAdjust(let message),
            .NoStackedImage(let message), .NoProcessedImage(let message), .NoFramesSelected(let message),
            .SortNeedsAlign(let message), .Cancelled(let message), .BadImageSize(let message),
            .Poisoned(let message), .Failed(let message):
            message
        }
    }
}

/// One open recording and the work built on it, over the Rust session (`crates/session` through `crates/ffi`).
///
/// The calls are the steps of the workflow. Everything that can take more than a moment is `async` and runs on a
/// background queue, so the main actor stays free; `cancel` and `pause` are synchronous and can be called meanwhile.
/// Cancelling the Swift `Task` of a run with progress (Align, Stack, Do All, Save Aligned/Integrated) cancels the run;
/// it then throws `PlanetsError.Cancelled`. Errors carry the text of the session in `PlanetsError.message`.
public final class PlanetsSession: Sendable {
    private let core: CoreSession

    /// A session without a recording that calculates on all logical cores.
    public init() throws {
        core = try CoreSession()
    }

    // MARK: Open

    public func openRecording(_ urls: [URL]) async throws -> RecordingInfo {
        let paths = urls.map(\.path)
        return try await run { try $0.openRecording(paths: paths) }
    }

    /// One frame of the open recording (0-based).
    public func readFrame(_ index: UInt32) async throws -> RgbaImage {
        try await run { try $0.readFrame(index: index) }
    }

    // MARK: Align

    /// Sets alignpoints on one frame with the default threshold. Starts a new alignment: results built on the old one are gone.
    public func setAlignpoints(frame: UInt32, _ settings: PointSettings) async throws -> Alignpoints {
        try await run { try $0.setAlignpoints(frame: frame, settings: settings) }
    }

    /// Moves the *Threshold* slider.
    public func setAlignpointThreshold(_ threshold: UInt64) throws -> Alignpoints {
        try core.setAlignpointThreshold(threshold: threshold)
    }

    public func align(_ settings: AlignSettings, progress: ProgressHandler? = nil) async throws -> Aligned {
        let listener = Listener(progress)
        return try await run(cancelling: .align) { try $0.align(settings: settings, progress: listener) }
    }

    public func limit(_ settings: LimitSettings) throws -> Limited {
        try core.limit(settings: settings)
    }

    // MARK: Stack

    public func stack(_ settings: StackSettings, progress: ProgressHandler? = nil) async throws -> Stacked {
        let listener = Listener(progress)
        return try await run(cancelling: .stack) { try $0.stack(settings: settings, progress: listener) }
    }

    public func stackedImage() async throws -> RgbaImage {
        try await run { try $0.stackedImage() }
    }

    // MARK: Wavelets

    public func waveletSetSourceFrame(_ index: UInt32) async throws -> SourceInfo {
        try await run { try $0.waveletSetSourceFrame(index: index) }
    }

    /// One area of the preview; `settings` of `nil` gives the unprocessed image (*Before*).
    public func waveletPreview(_ settings: WaveletSettings?, area: Region, mode: ColourMode) async throws -> WaveletPreview {
        try await run { try $0.waveletPreview(settings: settings, area: area, mode: mode) }
    }

    public func waveletDoAll(
        _ settings: WaveletSettings, mode: ColourMode, progress: ProgressHandler? = nil
    ) async throws -> RgbaImage {
        let listener = Listener(progress)
        return try await run(cancelling: .wavelet) { try $0.waveletDoAll(settings: settings, mode: mode, progress: listener) }
    }

    // MARK: Colour and tone

    public func setColorSource(_ image: RgbaImage) async throws -> SourceInfo {
        try await run { try $0.setColorSource(width: image.width, height: image.height, rgba: image.rgba) }
    }

    public func estimateRgbAlign(maxShift: UInt32) async throws -> ChannelShifts {
        try await run { try $0.estimateRgbAlign(maxShift: maxShift) }
    }

    public func applyColors(_ settings: ColorSettings) async throws -> RgbaImage {
        try await run { try $0.applyColors(settings: settings) }
    }

    /// The preview of Masking: the colour source with the part the sharpening applies to tinted blue.
    public func refineMaskOverlay(_ settings: RefineSettings) async throws -> RgbaImage {
        try await run { try $0.refineMaskOverlay(settings: settings) }
    }

    public func colorHistogram(_ settings: ColorSettings, after: Bool) async throws -> HistogramData {
        try await run { try $0.colorHistogram(settings: settings, after: after) }
    }

    public func autoHistogramPoints(_ settings: ColorSettings, clip: Float) async throws -> HistogramPoints {
        try await run { try $0.autoHistogramPoints(settings: settings, clip: clip) }
    }

    // MARK: Save

    /// Throws if there is no image of `source` to save, so the app can fail before it opens a file dialog.
    public func requireImage(_ source: ImageSource) throws {
        try core.requireImage(source: source)
    }

    /// Writes the image of `source` in `format`, with `adjustments` as the colour panels show it and cut to `crop` if there is one.
    /// Returns the file that was written (the extension is corrected to match the format).
    public func saveImage(
        _ source: ImageSource, format: ExportFormat, to url: URL, adjustments: ColorSettings? = nil, crop: CropRect? = nil
    ) async throws -> URL {
        let path = url.path
        let written = try await run {
            try $0.saveImage(source: source, format: format, path: path, adjustments: adjustments, crop: crop)
        }
        return URL(fileURLWithPath: written)
    }

    public func saveAligned(
        to url: URL, _ settings: SequenceSettings, progress: ProgressHandler? = nil
    ) async throws -> SavedVideo {
        let path = url.path
        let listener = Listener(progress)
        return try await run(cancelling: .sequence) {
            try $0.saveAligned(path: path, settings: settings, progress: listener)
        }
    }

    public func saveIntegrated(
        to url: URL, _ settings: SequenceSettings, progress: ProgressHandler? = nil
    ) async throws -> SavedVideo {
        let path = url.path
        let listener = Listener(progress)
        return try await run(cancelling: .sequence) {
            try $0.saveIntegrated(path: path, settings: settings, progress: listener)
        }
    }

    // MARK: Cancel, Pause, CPUs

    /// Stops a running operation; it then throws `PlanetsError.Cancelled`. Without a running one the Cancel is forgotten
    /// when the next one starts. `.sequence` shares the flag with `.stack`.
    public func cancel(_ operation: SessionOperation) {
        core.cancel(operation: operation)
    }

    public func pause(_ operation: SessionOperation, _ paused: Bool) {
        core.pause(operation: operation, paused: paused)
    }

    /// The number of logical cores: the default and the upper end of the *CPUs* field.
    public var cpuCount: Int { Int(core.cpuCount()) }

    /// The number of threads the calculations use now.
    public var cpus: Int { Int(core.cpus()) }

    /// *CPUs*: the calculations use this many threads from now on. Returns the number in use (clamped to 1...`cpuCount`).
    @discardableResult
    public func setCpus(_ count: UInt32) throws -> Int {
        Int(try core.setCpus(count: count))
    }

    // MARK: Off the main thread

    /// Runs a blocking call of the core on a background queue. A queue instead of the cooperative pool, because the call
    /// blocks for as long as the work (or a pause) lasts. With `operation`, cancelling the task cancels that operation.
    private func run<T: Sendable>(
        cancelling operation: SessionOperation? = nil,
        _ work: @escaping @Sendable (CoreSession) throws -> T
    ) async throws -> T {
        try Task.checkCancellation()
        let core = self.core
        return try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                DispatchQueue.global(qos: .userInitiated).async {
                    continuation.resume(with: Result { try work(core) })
                }
            }
        } onCancel: {
            if let operation { core.cancel(operation: operation) }
        }
    }
}

/// Adapts a Swift closure to the callback interface of the core.
private final class Listener: ProgressListener {
    private let handler: ProgressHandler?

    init(_ handler: ProgressHandler?) {
        self.handler = handler
    }

    func progress(operation: SessionOperation, done: UInt32, total: UInt32, elapsedMs: UInt64) {
        handler?(OperationProgress(
            operation: operation, done: done, total: total, elapsed: .milliseconds(elapsedMs)
        ))
    }
}
