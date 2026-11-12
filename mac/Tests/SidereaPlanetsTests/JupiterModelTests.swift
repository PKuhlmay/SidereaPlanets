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

/// Alignpoints, Align and Limit of the app model on the real Rust core and the real recording, as the window runs them with the
/// default values.
@MainActor
@Suite("JUPITER.AVI through the app model", .enabled(if: jupiter != nil, "JUPITER.AVI not found in re-local/"))
struct JupiterModelTests {
    private func makeModel() throws -> AppModel {
        let defaults = try #require(UserDefaults(suiteName: "JupiterModelTests-\(UUID().uuidString)"))
        return AppModel(backend: try PlanetsSession(), defaults: defaults)
    }

    @Test func alignpointsAlignAndLimitWithTheDefaults() async throws {
        let model = try makeModel()
        await model.open([jupiter!])
        #expect(model.frameCount == 10)

        // The points of the default settings: 78 after the threshold, 10 % either way (docs/testdaten.md).
        await model.setAlignpoints()
        #expect((71...85).contains(model.align.points.count), "\(model.align.points.count) points")
        #expect(model.align.total >= model.align.points.count)
        #expect(model.progress.step == .points && model.viewStep == .align)

        // Align rates every frame; frame 1 is the best one.
        await model.runAlign()
        #expect(model.progress.step == .aligned && model.viewStep == .limit)
        #expect(model.align.frames.allSatisfy { $0.quality != nil })
        #expect(model.align.listed.first?.index == 0, "frame 1 is the best one")
        #expect(model.align.frames[0].quality == 100)
        // The order of the Windows clone (docs/testdaten.md): the same core, so the same Framelist.
        #expect(model.align.listed.map(\.index) == [0, 2, 9, 1, 7, 8, 3, 4, 5, 6])
        #expect(model.status.result == "Alignment finished.")
        #expect(model.status.action == "Loading frame: 10")

        // Limit at the default (Lowest Quality 80 %) keeps all ten frames.
        await model.runLimit()
        #expect(model.progress.step == .limited && model.viewStep == .stack)
        #expect(model.align.keptCount == 10)
        #expect(model.status.action == "Processing limited to 10 frames")
        #expect(model.status.result.hasPrefix("Stacksize: 10  Low quality: "))
        #expect(model.result(of: .limit) == "10 of 10 kept")
    }

    @Test func stackOfTheDefaultsHas1024x680AndTenFramesAndAReAlignTakesItBack() async throws {
        let model = try makeModel()
        await model.open([jupiter!])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        await model.runStack()

        // The reference case of docs/testdaten.md: 10 frames of 1024x680.
        #expect(model.progress.step == .stacked && model.viewStep == .wavelets)
        #expect(model.stacked?.frames == 10 && model.stacked?.width == 1024 && model.stacked?.height == 680)
        #expect(model.stacked?.image.width == 1024 && model.stacked?.image.height == 680)
        #expect(model.status.action == "Stack size (n=10) - (1024x680)")
        #expect(model.status.result == "Stacking Range : 0-255")
        #expect(model.result(of: .stack) == "10 frames · 1024×680")
        model.show(.stack)
        #expect(model.imageCaption == "Stack of 10 frames")

        // Aligning again drops the stack in the core as well: asking for it fails.
        await model.runAlign()
        #expect(model.stacked == nil && model.progress.step == .aligned)
        await #expect(throws: PlanetsError.self) { _ = try await model.backend.stackImage() }
    }

    @Test func cancellingARealAlignLeavesTheStepReadyToRunAgain() async throws {
        // With one CPU the frames are rated one after the other; on all cores the ten frames of this recording
        // would be done within one batch, before the first progress reaches this test.
        let session = try PlanetsSession()
        try session.setCpus(1)
        let defaults = try #require(UserDefaults(suiteName: "JupiterModelTests-\(UUID().uuidString)"))
        let model = AppModel(backend: session, defaults: defaults)
        await model.open([jupiter!])
        await model.setAlignpoints()

        let running = Task { await model.runAlign() }
        // Stop it as soon as the first frame is rated: nine more are still to come.
        for _ in 0..<2000 where model.status.progress == 0 { try await Task.sleep(for: .milliseconds(5)) }
        #expect(model.status.progress > 0 && model.canCancel)
        model.cancelRun()
        await running.value

        #expect(model.status.result == "Cancelled")
        #expect(model.run == nil && !model.isBusy && model.status.progress == 0)
        #expect(model.progress.step == .points && !model.align.isRated)

        // The next run is not stopped by the old Cancel.
        await model.runAlign()
        #expect(model.progress.step == .aligned && model.status.result == "Alignment finished.")
    }
}
