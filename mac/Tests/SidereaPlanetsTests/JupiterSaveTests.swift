import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// JUPITER.AVI from `re-local/testdata/astropix/` (not in Git, see docs/testdaten.md): `$PLANETS_RE_LOCAL`, or the `re-local/` of
/// this repository. `nil` if the video is not there; the tests are skipped then.
private let jupiter: URL? = {
    let repoRoot = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent()  // SidereaPlanetsTests
        .deletingLastPathComponent()  // Tests
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

/// The whole way of the app model on the real Rust core and the real recording, down to the files: alignpoints, align, limit, stack, then
/// Save image as TIFF 16 bit, Save Aligned and the project file.
@MainActor
@Suite("JUPITER.AVI saved through the app model", .enabled(if: jupiter != nil, "JUPITER.AVI not found in re-local/"))
struct JupiterSaveTests {
    private func makeModel() throws -> AppModel {
        let defaults = try #require(UserDefaults(suiteName: "JupiterSaveTests-\(UUID().uuidString)"))
        return AppModel(backend: try PlanetsSession(), defaults: defaults)
    }

    private func tempFolder() throws -> URL {
        let folder = FileManager.default.temporaryDirectory.appendingPathComponent("planets-jupiter-save-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        return folder
    }

    private func stackedModel() async throws -> AppModel {
        let model = try makeModel()
        await model.open([jupiter!])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        await model.runStack()
        #expect(model.progress.step == .stacked)
        return model
    }

    @Test func theStackIsWrittenAsA16BitTiffAndAReAlignLocksSaveImage() async throws {
        let model = try await stackedModel()
        let folder = try tempFolder()
        model.show(.stack)
        let tiff = try #require(model.formats(for: .stack).first { $0.id == .tiff16 })
        let dialogs = Dialogs(answer: folder.appendingPathComponent("JUPITER.tif"))
        await model.saveImage(tiff, choose: dialogs.save)

        #expect(model.status.result == "Saved JUPITER.tif", "\(model.status.result)")
        let data = try Data(contentsOf: folder.appendingPathComponent("JUPITER.tif"))
        // Little endian ("II", 42) or big endian ("MM", 42) TIFF header.
        let magic = [UInt8](data.prefix(4))
        #expect(magic == [0x49, 0x49, 0x2A, 0x00] || magic == [0x4D, 0x4D, 0x00, 0x2A])
        // 1024 x 680 pixels with at least one 16 bit channel.
        #expect(data.count >= 1024 * 680 * 2)

        // The format wins over the typed extension.
        await model.saveImage(tiff, choose: Dialogs(answer: folder.appendingPathComponent("again.png")).save)
        #expect(model.status.result == "Saved again.tif")

        await model.runAlign()
        #expect(!model.canSaveImage && model.stacked == nil)
        #expect(throws: PlanetsError.self) { try model.backend.requireImage(.stack) }
    }

    @Test func alignedFramesAndTheProjectOfARealSession() async throws {
        let model = try await stackedModel()
        let folder = try tempFolder()
        model.stackOptions.framerate = 10
        await model.saveVideo(.aligned, choose: Dialogs(answer: folder.appendingPathComponent("J_aligned")).save)
        #expect(model.status.result == "Saved J_aligned.avi (10 frames)", "\(model.status.result)")
        let video = try Data(contentsOf: folder.appendingPathComponent("J_aligned.avi"))
        #expect([UInt8](video.prefix(4)) == Array("RIFF".utf8))

        model.options.boxSize = 41
        model.stackOptions.nearestPoints = 4
        let file = folder.appendingPathComponent("JUPITER.rkproj")
        await model.saveProject(choose: Dialogs(answer: file).save)
        #expect(model.status.result == "Saved JUPITER.rkproj", "\(model.status.result)")

        let other = try makeModel()
        await other.openProject(choose: Dialogs(answer: file).open)
        #expect(other.fileURLs.map(\.path) == [jupiter!.path] && other.frameCount == 10)
        #expect(other.options.boxSize == 41 && other.stackOptions.nearestPoints == 4 && other.stackOptions.framerate == 10)
        #expect(other.status.result == "Opened JUPITER.rkproj")
    }
}
