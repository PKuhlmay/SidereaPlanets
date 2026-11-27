import CoreGraphics
import Foundation
import Testing

@testable import SidereaPlanets

/// The status bar fields of MAC-022: the pixel under the pointer and the version of the build.
@Suite struct StatusBarTests {
    let image = CGSize(width: 1000, height: 500)

    @Test func thePixelUnderThePointerFollowsZoomAndPan() {
        let t = ViewTransform(scale: 2, x: 40, y: 30)
        #expect(t.pixel(at: CGPoint(x: 40, y: 30), image: image) == ImagePixel(x: 0, y: 0))
        // Between pixels the one the point lies in counts, as Math.floor in viewport.ts.
        #expect(t.pixel(at: CGPoint(x: 441.9, y: 431), image: image) == ImagePixel(x: 200, y: 200))
        #expect(t.pixel(at: CGPoint(x: 2039.9, y: 1029.9), image: image) == ImagePixel(x: 999, y: 499))
    }

    @Test func outsideTheImageThereIsNoPixel() {
        let t = ViewTransform(scale: 2, x: 40, y: 30)
        #expect(t.pixel(at: CGPoint(x: 39.9, y: 100), image: image) == nil)
        #expect(t.pixel(at: CGPoint(x: 100, y: 29.9), image: image) == nil)
        #expect(t.pixel(at: CGPoint(x: 2040, y: 100), image: image) == nil)
        #expect(t.pixel(at: CGPoint(x: 100, y: 1030), image: image) == nil)
        #expect(ViewTransform().pixel(at: .zero, image: .zero) == nil)
    }

    @Test func thePositionTextNamesThePixelAndTheStack() {
        #expect(pointerPositionText(ImagePixel(x: 200, y: 201), stackFrames: 10) == "X=200 Y=201 Stack=10")
        #expect(pointerPositionText(ImagePixel(x: 0, y: 7), stackFrames: nil) == "X=0 Y=7")
        #expect(pointerPositionText(nil, stackFrames: 10) == "")
    }

    @Test func theBuildLabelLeavesOutWhatIsNotKnown() throws {
        let utc = try #require(TimeZone(identifier: "UTC"))
        let built = try #require(ISO8601DateFormatter().date(from: "2026-11-13T13:38:59Z"))
        #expect(buildLabel(version: "0.1.0", commit: "a4e826b", builtAt: built, timeZone: utc) == "0.1.0 · a4e826b · 2026-11-13 13:38")
        #expect(buildLabel(version: "0.1.0", commit: nil, builtAt: built, timeZone: utc) == "0.1.0 · 2026-11-13 13:38")
        #expect(buildLabel(version: "0.1.0", commit: "", builtAt: nil, timeZone: utc) == "0.1.0")
        // Local time: the same moment in Berlin is an hour later.
        let berlin = try #require(TimeZone(identifier: "Europe/Berlin"))
        #expect(buildLabel(version: "0.1.0", commit: "a4e826b", builtAt: built, timeZone: berlin) == "0.1.0 · a4e826b · 2026-11-13 14:38")
    }
}

/// The position field in the app model, on the fake core: only while the sharpened stack is shown, and empty again in another step.
@MainActor
@Suite struct StatusBarModelTests {
    let backend = FakeBackend()
    let defaults: UserDefaults
    let model: AppModel

    init() throws {
        defaults = try #require(UserDefaults(suiteName: "StatusBarModelTests-\(UUID().uuidString)"))
        model = AppModel(backend: backend, defaults: defaults)
    }

    @Test func thePositionShowsInWaveletsAndSaveOnly() async {
        await model.open([URL(fileURLWithPath: "/data/JUPITER.AVI")])
        // Before the stack the field stays empty: the frame steps have no position.
        model.pointerMoved(to: ImagePixel(x: 1, y: 2))
        #expect(model.status.position == "")

        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        await model.runStack()
        await model.settleWaveletPreview()
        #expect(model.viewStep == .wavelets)
        model.pointerMoved(to: ImagePixel(x: 4, y: 3))
        #expect(model.status.position == "X=4 Y=3 Stack=3")
        model.pointerMoved(to: nil)
        #expect(model.status.position == "")

        model.pointerMoved(to: ImagePixel(x: 5, y: 1))
        model.show(.stack)
        #expect(model.status.position == "")
        model.pointerMoved(to: ImagePixel(x: 5, y: 1))
        #expect(model.status.position == "")
    }
}
