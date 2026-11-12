import CoreGraphics
import Foundation
import SidereaPlanetsKit

/// What the app model needs from the Rust core. `PlanetsSession` is the real one; the tests bring a fake. Later steps
/// add their calls here.
protocol SessionBackend: Sendable {
    func openRecording(_ urls: [URL]) async throws -> RecordingInfo
    /// One frame of the open recording (0-based), ready for display.
    func frameImage(_ index: UInt32) async throws -> CGImage
}

struct FrameConversionError: Error, LocalizedError {
    var errorDescription: String? { "the frame could not be turned into an image" }
}

extension PlanetsSession: SessionBackend {
    /// Reads the frame on the core's queue and builds the `CGImage` off the main actor too.
    @concurrent
    func frameImage(_ index: UInt32) async throws -> CGImage {
        let frame = try await readFrame(index)
        guard let image = frame.makeCGImage() else { throw FrameConversionError() }
        return image
    }
}

/// The text the window shows for an error of the core or of the system.
func errorText(_ error: Error) -> String {
    (error as? PlanetsError)?.message ?? error.localizedDescription
}
