import SidereaPlanetsKit
import SwiftUI

/// Marks drawn over the image in image pixels, so they zoom with it (docs/design.md, "Bildbereich"): the alignpoints as red dots
/// after Set Alignpoints, as yellow rings with a green shift stroke in the Stack step, the region of interest as a dashed amber
/// rectangle, and the rectangle that is being drawn. Never in the way of the mouse.
struct ImageOverlays: View {
    let transform: ViewTransform
    let points: [Point]
    /// The rings of the Stack step.
    var sections: [AlignSection] = []
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
            for section in sections {
                let center = CGPoint(x: Double(section.point.x) * scale + transform.x, y: Double(section.point.y) * scale + transform.y)
                let r = 7 * scale
                context.stroke(
                    Path(ellipseIn: CGRect(x: center.x - r, y: center.y - r, width: 2 * r, height: 2 * r)),
                    with: .color(RK.alignpointRing), lineWidth: 1.4
                )
                // The shift is only a few pixels, so the stroke shows it enlarged to stay visible.
                if let shift = section.shift {
                    var stroke = Path()
                    stroke.move(to: center)
                    stroke.addLine(to: CGPoint(x: center.x + Double(shift.dx) * 4 * scale, y: center.y + Double(shift.dy) * 4 * scale))
                    context.stroke(stroke, with: .color(RK.alignpointShift), lineWidth: 1.4)
                }
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
