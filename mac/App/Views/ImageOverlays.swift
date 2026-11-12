import SidereaPlanetsKit
import SwiftUI

/// Marks drawn over the image in image pixels, so they zoom with it (docs/design.md, "Bildbereich"): the alignpoints as red dots
/// after Set Alignpoints, the region of interest as a dashed amber rectangle, and the rectangle that is being drawn. Never in the way
/// of the mouse.
struct ImageOverlays: View {
    let transform: ViewTransform
    let points: [Point]
    let region: RoiSettings?
    let draft: RoiSettings?

    var body: some View {
        Canvas { context, _ in
            let scale = transform.scale
            for point in points {
                let center = CGPoint(x: Double(point.x) * scale + transform.x, y: Double(point.y) * scale + transform.y)
                let r = 3.2 * scale
                context.fill(Path(ellipseIn: CGRect(x: center.x - r, y: center.y - r, width: 2 * r, height: 2 * r)), with: .color(RK.alignpoint))
            }
            for rect in [region, draft].compactMap({ $0 }) {
                let frame = CGRect(
                    x: Double(rect.x) * scale + transform.x, y: Double(rect.y) * scale + transform.y,
                    width: Double(rect.width) * scale, height: Double(rect.height) * scale
                )
                context.stroke(Path(frame), with: .color(RK.accent), style: StrokeStyle(lineWidth: 1.5, dash: [6, 5]))
            }
        }
        .allowsHitTesting(false)
        .accessibilityHidden(true)
    }
}
