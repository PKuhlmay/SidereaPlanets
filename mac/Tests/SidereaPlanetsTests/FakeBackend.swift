import CoreGraphics
import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// What the synchronous calls (threshold, cancel, pause) leave behind; they are called from the main actor while a run is held back in
/// the actor, so they cannot live in it.
final class CallLog: @unchecked Sendable {
    private let lock = NSLock()
    private var thresholdValue = 0
    private var cancelled: [SessionOperation] = []
    private var pauses: [Bool] = []
    private var cancelHandler: (@Sendable () -> Void)?
    private var imageError: String?

    func setImageError(_ message: String?) { lock.withLock { imageError = message } }
    var missingImage: String? { lock.withLock { imageError } }
    func setThreshold(_ value: Int) { lock.withLock { thresholdValue = value } }
    var threshold: Int { lock.withLock { thresholdValue } }
    var cancelledOperations: [SessionOperation] { lock.withLock { cancelled } }
    var pauseCalls: [Bool] { lock.withLock { pauses } }
    func onCancel(_ handler: @escaping @Sendable () -> Void) { lock.withLock { cancelHandler = handler } }

    func recordCancel(_ operation: SessionOperation) {
        let handler = lock.withLock { () -> (@Sendable () -> Void)? in
            cancelled.append(operation)
            return cancelHandler
        }
        handler?()
    }

    func recordPause(_ paused: Bool) { lock.withLock { pauses.append(paused) } }
}

/// A stand-in for the Rust core: opens whatever it is given, answers frames with a (n+1) x 1 image and can hold frame reads back
/// so a test decides when each one finishes. Alignpoints, Align and Limit answer with the values set on it.
actor FakeBackend: SessionBackend {
    struct Failure: Error, LocalizedError {
        let message: String
        var errorDescription: String? { message }
    }

    static func info(frames: UInt32 = 10) -> RecordingInfo {
        RecordingInfo(
            format: "AVI", codec: "DIB", width: 2, height: 2, bitDepth: 8, frameCount: frames, color: "Rgb", fps: 25,
            observer: nil, instrument: nil, telescope: nil
        )
    }

    var recording = FakeBackend.info()
    var openError: String?
    /// While true, `frameImage` waits for `releaseOneRead()`.
    var holdsReads = false
    private(set) var requestedFrames: [UInt32] = []
    private var waiting: [CheckedContinuation<Void, Never>] = []

    func setHoldsReads(_ hold: Bool) { holdsReads = hold }
    func setOpenError(_ message: String?) { openError = message }
    func setRecording(_ info: RecordingInfo) { recording = info }

    func openRecording(_ urls: [URL]) async throws -> RecordingInfo {
        if let openError { throw Failure(message: openError) }
        return recording
    }

    func frameImage(_ index: UInt32) async throws -> CGImage {
        requestedFrames.append(index)
        if holdsReads {
            await withCheckedContinuation { waiting.append($0) }
        }
        // The width tells which frame an image is: frame n is n + 1 pixels wide.
        let width = Int(index) + 1
        let rgba = RgbaImage(width: UInt32(width), height: 1, rgba: Data(count: width * 4))
        return try #require(rgba.makeCGImage())
    }

    // MARK: Alignpoints, Align, Limit

    nonisolated let log = CallLog()
    /// Eight candidate points; the default threshold drops the two weakest.
    static let candidateCount = 8
    static let defaultThreshold = 2
    var noPoints = false
    /// The quality of each frame Align reports (10 frames, as the recording).
    var qualities: [Float] = [92.1, 88.4, 97.3, 85.0, 90.6, 99.2, 83.7, 95.8, 87.2, 100.0]
    /// The frames Limit keeps, best first.
    var keptFrames: [UInt32] = [9, 5, 2]
    /// How far each point moved, as Limit reports it.
    var pointShifts: [PointShift?] = []
    var alignError: String?
    var holdsAlign = false
    private(set) var pointCalls: [(frame: UInt32, settings: PointSettings)] = []
    private(set) var alignCalls: [AlignSettings] = []
    private(set) var limitCalls: [LimitSettings] = []
    private var alignWaiter: CheckedContinuation<Void, Error>?

    func setNoPoints(_ value: Bool) { noPoints = value }
    func setAlignError(_ message: String?) { alignError = message }
    func setHoldsAlign(_ hold: Bool) { holdsAlign = hold }
    func setKeptFrames(_ frames: [UInt32]) { keptFrames = frames }
    func setPointShifts(_ shifts: [PointShift?]) { pointShifts = shifts }
    var isAlignHeld: Bool { alignWaiter != nil }

    private static func points(_ count: Int) -> [Point] {
        (0..<count).map { Point(x: UInt32(20 * ($0 + 1)), y: UInt32(10 * ($0 + 1))) }
    }

    func setAlignpoints(frame: UInt32, _ settings: PointSettings) async throws -> Alignpoints {
        pointCalls.append((frame, settings))
        if noPoints { return Alignpoints(points: [], total: 0, threshold: 0) }
        log.setThreshold(Self.defaultThreshold)
        return Alignpoints(
            points: Self.points(Self.candidateCount - Self.defaultThreshold), total: UInt64(Self.candidateCount),
            threshold: UInt64(Self.defaultThreshold)
        )
    }

    nonisolated func setAlignpointThreshold(_ threshold: UInt64) throws -> Alignpoints {
        log.setThreshold(Int(threshold))
        return Alignpoints(
            points: Self.points(max(Self.candidateCount - Int(threshold), 0)), total: UInt64(Self.candidateCount), threshold: threshold
        )
    }

    func align(_ settings: AlignSettings, progress: ProgressHandler?) async throws -> Aligned {
        alignCalls.append(settings)
        if let alignError { throw Failure(message: alignError) }
        let total = UInt32(qualities.count)
        for done in 1...max(total / 2, 1) {
            progress?(OperationProgress(operation: .align, done: done, total: total, elapsed: .milliseconds(Int64(done) * 100)))
        }
        if holdsAlign {
            log.onCancel { [weak self] in Task { await self?.failHeldAlign() } }
            try await withCheckedThrowingContinuation { alignWaiter = $0 }
        }
        progress?(OperationProgress(operation: .align, done: total, total: total, elapsed: .seconds(1)))
        return Aligned(
            frames: qualities.enumerated().map {
                FrameRating(index: UInt32($0.offset), quality: $0.element, shiftX: 0, shiftY: 0, difference: Float($0.offset))
            },
            ranking: qualities.enumerated().sorted { $0.element > $1.element }.map { UInt32($0.offset) }
        )
    }

    /// Lets a held Align finish.
    func finishAlign() {
        alignWaiter?.resume()
        alignWaiter = nil
    }

    private func failHeldAlign() {
        alignWaiter?.resume(throwing: PlanetsError.Cancelled(message: "cancelled"))
        alignWaiter = nil
    }

    func limit(_ settings: LimitSettings) async throws -> Limited {
        limitCalls.append(settings)
        return Limited(frames: keptFrames, pointShifts: pointShifts)
    }

    // MARK: Stack

    /// The picture of the stack is 7 x 1 pixels, so a test can tell it from a frame (frame n is n + 1 pixels wide: frame 7 is 8).
    static let stackPictureWidth = 7
    var stackError: String?
    var holdsStack = false
    /// What the core reports as the stack size.
    var stackSize = (width: 1024, height: 680)
    private(set) var stackCalls: [StackSettings] = []
    private(set) var stackImageCalls = 0
    private var stackWaiter: CheckedContinuation<Void, Error>?

    func setStackError(_ message: String?) { stackError = message }
    func setHoldsStack(_ hold: Bool) { holdsStack = hold }
    var isStackHeld: Bool { stackWaiter != nil }

    func stack(_ settings: StackSettings, progress: ProgressHandler?) async throws -> Stacked {
        stackCalls.append(settings)
        if let stackError { throw Failure(message: stackError) }
        let total = UInt32(settings.frames.count)
        for done in 1...max(total / 2, 1) {
            progress?(OperationProgress(operation: .stack, done: done, total: total, elapsed: .milliseconds(Int64(done) * 100)))
        }
        if holdsStack {
            log.onCancel { [weak self] in Task { await self?.failHeldStack() } }
            try await withCheckedThrowingContinuation { stackWaiter = $0 }
        }
        progress?(OperationProgress(operation: .stack, done: total, total: total, elapsed: .seconds(1)))
        return Stacked(width: UInt32(stackSize.width), height: UInt32(stackSize.height), channels: 3, frames: total, elapsedMs: 1000)
    }

    func stackImage() async throws -> CGImage {
        stackImageCalls += 1
        let rgba = RgbaImage(width: UInt32(Self.stackPictureWidth), height: 1, rgba: Data(count: Self.stackPictureWidth * 4))
        return try #require(rgba.makeCGImage())
    }

    /// Lets a held Stack finish.
    func finishStack() {
        stackWaiter?.resume()
        stackWaiter = nil
    }

    private func failHeldStack() {
        stackWaiter?.resume(throwing: PlanetsError.Cancelled(message: "cancelled"))
        stackWaiter = nil
    }

    // MARK: Save

    struct ImageSave {
        var source: ImageSource
        var format: ExportFormat
        var url: URL
        var adjustments: ColorSettings?
    }

    struct VideoSave {
        var integrated: Bool
        var url: URL
        var settings: SequenceSettings
    }

    var saveError: String?
    var holdsVideo = false
    private(set) var imageSaves: [ImageSave] = []
    private(set) var videoSaves: [VideoSave] = []
    private var videoWaiter: CheckedContinuation<Void, Error>?

    func setSaveError(_ message: String?) { saveError = message }
    func setHoldsVideo(_ hold: Bool) { holdsVideo = hold }
    var isVideoHeld: Bool { videoWaiter != nil }

    nonisolated func requireImage(_ source: ImageSource) throws {
        if let message = log.missingImage { throw Failure(message: message) }
    }

    func saveImage(_ source: ImageSource, format: ExportFormat, to url: URL, adjustments: ColorSettings?) async throws -> URL {
        imageSaves.append(ImageSave(source: source, format: format, url: url, adjustments: adjustments))
        if let saveError { throw Failure(message: saveError) }
        return url
    }

    func saveAligned(to url: URL, _ settings: SequenceSettings, progress: ProgressHandler?) async throws -> SavedVideo {
        try await writeVideo(integrated: false, to: url, settings, progress: progress)
    }

    func saveIntegrated(to url: URL, _ settings: SequenceSettings, progress: ProgressHandler?) async throws -> SavedVideo {
        try await writeVideo(integrated: true, to: url, settings, progress: progress)
    }

    private func writeVideo(integrated: Bool, to url: URL, _ settings: SequenceSettings, progress: ProgressHandler?) async throws -> SavedVideo {
        videoSaves.append(VideoSave(integrated: integrated, url: url, settings: settings))
        if let saveError { throw Failure(message: saveError) }
        let total = UInt32(settings.stack.frames.count)
        progress?(OperationProgress(operation: .sequence, done: max(total / 2, 1), total: total, elapsed: .milliseconds(100)))
        if holdsVideo {
            log.onCancel { [weak self] in Task { await self?.failHeldVideo() } }
            try await withCheckedThrowingContinuation { videoWaiter = $0 }
        }
        progress?(OperationProgress(operation: .sequence, done: total, total: total, elapsed: .seconds(1)))
        return SavedVideo(path: url.path, frames: total)
    }

    /// Lets a held video finish.
    func finishVideo() {
        videoWaiter?.resume()
        videoWaiter = nil
    }

    private func failHeldVideo() {
        videoWaiter?.resume(throwing: PlanetsError.Cancelled(message: "cancelled"))
        videoWaiter = nil
    }

    nonisolated func cancel(_ operation: SessionOperation) { log.recordCancel(operation) }
    nonisolated func pause(_ operation: SessionOperation, _ paused: Bool) { log.recordPause(paused) }

    var waitingReads: Int { waiting.count }

    func releaseOneRead() {
        if !waiting.isEmpty { waiting.removeFirst().resume() }
    }

    /// Lets the read that started last finish first, so an older read comes back late.
    func releaseNewestRead() {
        if !waiting.isEmpty { waiting.removeLast().resume() }
    }
}
