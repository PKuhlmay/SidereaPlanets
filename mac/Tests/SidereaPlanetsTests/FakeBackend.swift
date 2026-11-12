import CoreGraphics
import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// A stand-in for the Rust core: opens whatever it is given, answers frames with a (n+1) x 1 image and can hold frame reads back
/// so a test decides when each one finishes.
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

    var waitingReads: Int { waiting.count }

    func releaseOneRead() {
        if !waiting.isEmpty { waiting.removeFirst().resume() }
    }

    /// Lets the read that started last finish first, so an older read comes back late.
    func releaseNewestRead() {
        if !waiting.isEmpty { waiting.removeLast().resume() }
    }
}
