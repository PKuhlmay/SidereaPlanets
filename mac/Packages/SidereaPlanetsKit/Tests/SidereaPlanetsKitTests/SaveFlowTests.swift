import Foundation
import ImageIO
import Testing

@testable import SidereaPlanetsKit

/// JUPITER.AVI from `re-local/testdata/astropix/` (not in Git, see docs/testdaten.md): `$PLANETS_RE_LOCAL`, or the
/// `re-local/` of the repository this package lives in. `nil` if the video is not there; the tests are skipped then.
private let jupiter: URL? = {
    let repoRoot = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent()  // SidereaPlanetsKitTests
        .deletingLastPathComponent()  // Tests
        .deletingLastPathComponent()  // SidereaPlanetsKit
        .deletingLastPathComponent()  // Packages
        .deletingLastPathComponent()  // mac
        .deletingLastPathComponent()  // repository root
    var candidates = [repoRoot.appendingPathComponent("re-local")]
    if let override = ProcessInfo.processInfo.environment["PLANETS_RE_LOCAL"] {
        candidates.insert(URL(fileURLWithPath: override), at: 0)
    }
    return candidates
        .map { $0.appendingPathComponent("testdata/astropix/JUPITER.AVI") }
        .first { FileManager.default.fileExists(atPath: $0.path) }
}()

private func tempFolder() throws -> URL {
    let folder = FileManager.default.temporaryDirectory.appendingPathComponent("planets-kit-save-\(UUID().uuidString)")
    try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
    return folder
}

@Suite("Save without a recording")
struct SaveBasicsTests {
    @Test("Save image without a stack fails with the text of the session")
    func noImage() throws {
        let session = try PlanetsSession()
        do {
            try session.requireImage(.stack)
            Issue.record("there is no image to save")
        } catch let error as PlanetsError {
            #expect(error.message.contains("no stacked image"), "\(error.message)")
        }
    }

    @Test("The file type lists: seven for the stack, five for the sharpened image")
    func formats() {
        #expect(exportFormats(source: .stack).count == 7)
        #expect(exportFormats(source: .wavelet).map(\.id) == [.bmp, .jpeg, .fits16, .tiff16, .png16])
    }

    @Test("The project file keeps the recording and the sections, and refuses a newer layout")
    func project() throws {
        let folder = try tempFolder()
        let project = Project(
            formatVersion: 1, recording: ["/data/a.ser"], sections: ["align": #"{"boxSize":30}"#, "extra": "[1,2]"])
        let written = try projectSave(project: project, path: folder.appendingPathComponent("p").path)
        #expect(written.hasSuffix("p.sidereaplanets"))

        let loaded = try projectLoad(path: written)
        #expect(loaded.project.recording == ["/data/a.ser"])
        #expect(loaded.missingFiles == ["/data/a.ser"])
        #expect(loaded.project.sections.keys.sorted() == ["align", "extra"])

        let newer = folder.appendingPathComponent("newer.sidereaplanets")
        try #"{"formatVersion": 9, "recording": [], "sections": {}}"#.write(to: newer, atomically: true, encoding: .utf8)
        #expect(throws: PlanetsError.self) { _ = try projectLoad(path: newer.path) }
    }
}

@Suite("JUPITER.AVI saved through SidereaPlanetsKit", .enabled(if: jupiter != nil, "JUPITER.AVI not found in re-local/"))
struct JupiterSaveFlowTests {
    @Test("Alignpoints, align, limit, stack, then Save image as TIFF 16 bit; a new align takes the stack away")
    func tiff16() async throws {
        let session = try PlanetsSession()
        _ = try await session.openRecording([jupiter!])
        let points = try await session.setAlignpoints(
            frame: 0,
            PointSettings(minDistance: 30, edgeDistance: 20, maxMovement: 5, roi: nil, intensity: .default, lo: 30, hi: 230, boxSize: 30))
        let alignSettings = AlignSettings(boxSize: 30, maxMovement: 5, roi: nil)
        _ = try await session.align(alignSettings)
        let limited = try session.limit(.lowestQuality(value: 80))
        let stackSettings = StackSettings(
            referenceFrame: 0, points: points.points.map { PointPosition(x: $0.x, y: $0.y) }, frames: limited.frames,
            boxSize: 30, maxMovement: 5, nearestPoints: 3, correctGeometry: true, normalize: false, edgeSkip: 0, roi: nil)
        _ = try await session.stack(stackSettings)

        let folder = try tempFolder()
        try session.requireImage(.stack)
        let written = try await session.saveImage(.stack, format: .tiff16, to: folder.appendingPathComponent("JUPITER.tif"))
        #expect(written.lastPathComponent == "JUPITER.tif")
        let data = try Data(contentsOf: written)
        let magic = [UInt8](data.prefix(4))
        #expect(magic == [0x49, 0x49, 0x2A, 0x00] || magic == [0x4D, 0x4D, 0x00, 0x2A], "TIFF header")
        #expect(data.count >= 1024 * 680 * 2)

        // Turned by 90 degrees, cut (in the turned picture) and halved: 680 x 1024 -> 300 x 400 (MAC-017).
        let geometry = GeometrySettings(
            orientation: OrientationSettings(flipHorizontal: false, flipVertical: false, angle: 90, cropToFit: false, method: .bicubic),
            resize: ResizeSettings(target: .percent(percent: 50), method: .lanczos3))
        let turned = try await session.saveImage(
            .stack, format: .tiff16, to: folder.appendingPathComponent("turned.tif"),
            crop: CropRect(x: 40, y: 100, width: 600, height: 800), geometry: geometry)
        let source = try #require(CGImageSourceCreateWithURL(turned as CFURL, nil))
        let properties = try #require(CGImageSourceCopyPropertiesAtIndex(source, 0, nil) as? [CFString: Any])
        #expect(properties[kCGImagePropertyPixelWidth] as? Int == 300)
        #expect(properties[kCGImagePropertyPixelHeight] as? Int == 400)

        // Save Aligned: one video frame per kept frame, with progress.
        let sequence = SequenceSettings(stack: stackSettings, framerate: 25, sortOnQuality: false, averageFrames: 5)
        let video = try await session.saveAligned(to: folder.appendingPathComponent("J_aligned.avi"), sequence)
        #expect(video.frames == 10)
        #expect(try Data(contentsOf: URL(fileURLWithPath: video.path)).prefix(4) == Data("RIFF".utf8))

        // Aligning again drops the stack: there is nothing to save until it is stacked again.
        _ = try await session.align(alignSettings)
        #expect(throws: PlanetsError.self) { try session.requireImage(.stack) }
    }
}
