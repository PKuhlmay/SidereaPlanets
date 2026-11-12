import CoreGraphics
import Testing

@testable import SidereaPlanets

@Suite struct ViewTransformTests {
    let image = CGSize(width: 1000, height: 500)

    @Test func fitCentresTheImageWithAMargin() {
        let t = ViewTransform.fit(box: CGSize(width: 1224, height: 624), image: image)
        #expect(t.scale == 1.2)
        #expect(t.x == 12)
        #expect(t.y == 12)
        let wide = ViewTransform.fit(box: CGSize(width: 2000, height: 624), image: image)
        #expect(wide.scale == 1.2)
        #expect(wide.x == 400)
    }

    @Test func zoomKeepsThePointUnderThePointerInPlace() {
        let start = ViewTransform(scale: 1, x: 40, y: 30)
        let pointer = CGPoint(x: 300, y: 200)
        let zoomed = start.zoomed(by: 2, at: pointer)
        #expect(zoomed.scale == 2)
        // The image pixel under the pointer is the same before and after.
        let before = CGPoint(x: (pointer.x - start.x) / start.scale, y: (pointer.y - start.y) / start.scale)
        let after = CGPoint(x: (pointer.x - zoomed.x) / zoomed.scale, y: (pointer.y - zoomed.y) / zoomed.scale)
        #expect(before == after)
    }

    @Test func zoomStaysWithinTheLimits() {
        let t = ViewTransform(scale: 20)
        #expect(t.zoomed(by: 10, at: .zero).scale == ViewTransform.maxScale)
        #expect(t.zoomed(by: 0.00001, at: .zero).scale == ViewTransform.minScale)
    }

    @Test func actualSizeShowsOneScreenPointPerPixel() {
        let t = ViewTransform(scale: 0.4, x: 10, y: 20).actualSize(in: CGSize(width: 800, height: 600))
        #expect(t.scale == 1)
        #expect(ViewTransform(scale: 1.056).label == "106%")
    }
}
