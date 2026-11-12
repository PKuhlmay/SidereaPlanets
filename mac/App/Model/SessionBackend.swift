import CoreGraphics
import Foundation
import SidereaPlanetsKit

/// What the app model needs from the Rust core. `PlanetsSession` is the real one; the tests bring a fake. Later steps
/// add their calls here.
protocol SessionBackend: WaveletBackend {
    func openRecording(_ urls: [URL]) async throws -> RecordingInfo
    /// One frame of the open recording (0-based), ready for display.
    func frameImage(_ index: UInt32) async throws -> CGImage

    /// Sets alignpoints on one frame. Starts a new alignment: results built on the old one are gone.
    func setAlignpoints(frame: UInt32, _ settings: PointSettings) async throws -> Alignpoints
    /// The *Threshold* slider; synchronous, because it moves with every step of the slider.
    func setAlignpointThreshold(_ threshold: UInt64) throws -> Alignpoints
    func align(_ settings: AlignSettings, progress: ProgressHandler?) async throws -> Aligned
    func limit(_ settings: LimitSettings) async throws -> Limited
    func stack(_ settings: StackSettings, progress: ProgressHandler?) async throws -> Stacked
    /// The picture of the last stack, ready for display.
    func stackImage() async throws -> CGImage
    /// Stops a running operation; it then throws `PlanetsError.Cancelled`.
    func cancel(_ operation: SessionOperation)
    func pause(_ operation: SessionOperation, _ paused: Bool)

    /// Throws if there is no image of `source` to save, so the app can fail before it opens a file dialog.
    func requireImage(_ source: ImageSource) throws
    /// Writes the image of `source`, with `adjustments` as the colour panels show it; returns the file that was written.
    func saveImage(_ source: ImageSource, format: ExportFormat, to url: URL, adjustments: ColorSettings?) async throws -> URL
    func saveAligned(to url: URL, _ settings: SequenceSettings, progress: ProgressHandler?) async throws -> SavedVideo
    func saveIntegrated(to url: URL, _ settings: SequenceSettings, progress: ProgressHandler?) async throws -> SavedVideo
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

    @concurrent
    func stackImage() async throws -> CGImage {
        let stacked = try await stackedImage()
        guard let image = stacked.makeCGImage() else { throw FrameConversionError() }
        return image
    }
}

/// The text the window shows for an error of the core or of the system.
func errorText(_ error: Error) -> String {
    (error as? PlanetsError)?.message ?? error.localizedDescription
}
