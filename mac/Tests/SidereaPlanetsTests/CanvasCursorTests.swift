import CoreGraphics
import Testing
@testable import SidereaPlanets

@MainActor
struct CanvasCursorTests {
    private let bounds = CGRect(x: 0, y: 0, width: 800, height: 600)

    @Test func aCursorAreaOutsideTheViewAddsNoRect() {
        // A crop edge panned out of sight: AppKit throws on the null rect of a disjoint intersection.
        let offscreen = CGRect(x: 900, y: 100, width: 20, height: 200)
        #expect(ImageCanvasView.visibleCursorArea(offscreen, in: bounds) == nil)
    }

    @Test func aCursorAreaThatOnlyTouchesTheViewAddsNoRect() {
        let touching = CGRect(x: 800, y: 100, width: 20, height: 200)
        #expect(ImageCanvasView.visibleCursorArea(touching, in: bounds) == nil)
    }

    @Test func aCursorAreaPartlyInsideIsClippedToTheView() {
        let partly = CGRect(x: 780, y: 100, width: 40, height: 200)
        #expect(ImageCanvasView.visibleCursorArea(partly, in: bounds) == CGRect(x: 780, y: 100, width: 20, height: 200))
    }
}
