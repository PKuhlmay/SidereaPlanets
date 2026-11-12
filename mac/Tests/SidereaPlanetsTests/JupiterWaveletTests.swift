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

/// The Wavelets step of the app model on the real Rust core and the real recording: the stack of JUPITER.AVI sharpened with the default
/// values and the colour modes, and the cards.
@MainActor
@Suite("JUPITER.AVI sharpened through the app model", .enabled(if: jupiter != nil, "JUPITER.AVI not found in re-local/"))
struct JupiterWaveletTests {
    private func stackedModel() async throws -> AppModel {
        let defaults = try #require(UserDefaults(suiteName: "JupiterWaveletTests-\(UUID().uuidString)"))
        let model = AppModel(backend: try PlanetsSession(), defaults: defaults)
        await model.open([jupiter!])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        await model.runStack()
        await model.settleWaveletPreview()
        #expect(model.progress.step == .stacked && model.viewStep == .wavelets)
        return model
    }

    /// Mean absolute difference per channel value (0...255) of two pictures of the same size, and the number of values that differ.
    private func compare(_ a: RgbaImage, _ b: RgbaImage) -> (mean: Double, differing: Int) {
        var sum = 0
        var differing = 0
        for i in stride(from: 0, to: a.rgba.count, by: 4) {
            for c in 0..<3 {
                let d = abs(Int(a.rgba[i + c]) - Int(b.rgba[i + c]))
                sum += d
                if d != 0 { differing += 1 }
            }
        }
        return (Double(sum) / Double(a.rgba.count / 4 * 3), differing)
    }

    @Test func doAllWithTheDefaultsKeepsTheSizeAndChangesTheImageSlightly() async throws {
        let model = try await stackedModel()
        let stack = try #require(model.waveletData.stack?.pixels)
        #expect(stack.width == 1024 && stack.height == 680)

        await model.doAll()
        #expect(model.status.result == "Do All finished")
        #expect(model.progress.waveletsDone && model.viewStep == .save)
        let sharpened = try #require(model.waveletData.processed?.pixels)
        #expect(sharpened.width == 1024 && sharpened.height == 680)

        // Neutral layers still pass through the tone curve of the original (docs/wavelets.md): not the identity, but close to it.
        let (mean, differing) = compare(stack, sharpened)
        #expect(differing > 0, "the default settings left the image as it was")
        #expect(mean > 0 && mean < 6, "mean change \(mean)")
    }

    @Test func theLivePreviewEqualsDoAllInsideTheProcessingArea() async throws {
        let model = try await stackedModel()
        let area = try #require(model.processingAreaRegion)
        #expect(area == Region(x: 256, y: 84, width: 512, height: 512))
        let preview = try #require(model.waveletData.processed?.pixels)

        await model.doAll()
        let full = try #require(model.waveletData.processed?.pixels)
        var different = 0
        for y in Int(area.y)..<Int(area.y + area.height) {
            for x in Int(area.x)..<Int(area.x + area.width) {
                let i = (y * 1024 + x) * 4
                if preview.rgba[i..<i + 3] != full.rgba[i..<i + 3] { different += 1 }
            }
        }
        #expect(different == 0, "\(different) pixels of the area differ")
        // Outside it the preview still shows the stack without the tone curve.
        #expect(preview.rgba[0..<3] != full.rgba[0..<3] || preview.rgba[0] == 0)
    }

    @Test func aSharpenedLayerChangesMoreThanTheDefaultsAndGrayHasOneChannel() async throws {
        let model = try await stackedModel()
        let stack = try #require(model.waveletData.stack?.pixels)
        await model.doAll()
        let neutral = try #require(model.waveletData.processed?.pixels)

        model.show(.wavelets)
        model.moveLevel(0, to: 100)
        await model.settleWaveletPreview()
        await model.doAll()
        let sharp = try #require(model.waveletData.processed?.pixels)
        #expect(compare(stack, sharp).mean > compare(stack, neutral).mean)

        model.show(.wavelets)
        model.wavelet.colour = false
        await model.doAll()
        #expect(model.status.result == "Do All finished" && model.progress.waveletsDone, "\(model.status.result)")
        let gray = try #require(model.waveletData.processed?.pixels)
        let sample = stride(from: 0, to: gray.rgba.count, by: 4_000).allSatisfy { gray.rgba[$0] == gray.rgba[$0 + 1] && gray.rgba[$0] == gray.rgba[$0 + 2] }
        #expect(sample, "without Colour the result has no colour")
        #expect(!(stride(from: 0, to: sharp.rgba.count, by: 4_000).allSatisfy { sharp.rgba[$0] == sharp.rgba[$0 + 1] && sharp.rgba[$0] == sharp.rgba[$0 + 2] }))
    }

    @Test func theCardsChangeTheImageAndTheHistogramHas256Bins() async throws {
        let model = try await stackedModel()
        let base = try #require(model.waveletData.base?.pixels)
        model.tone.gamma = 1.8
        await model.settleColours()
        let adjusted = try #require(model.waveletData.adjusted?.pixels)
        #expect(adjusted.width == base.width && compare(base, adjusted).mean > 1)

        let histogram = try #require(await model.histogram(after: false))
        #expect(histogram.bins == 256 && histogram.luma.count == 256 && histogram.channels.count == 3)

        await model.autoLevels()
        #expect(model.tone.black >= 0 && model.tone.white <= 255 && model.tone.black < model.tone.white)
        await model.autoAlign()
        #expect(model.status.result == "RGB Align: shifts found")
        #expect(abs(model.tone.shifts.green.x) < 0.001 && abs(model.tone.shifts.red.x) <= 5.001)
    }
}
