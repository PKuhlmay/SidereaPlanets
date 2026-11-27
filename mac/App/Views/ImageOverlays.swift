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
    /// The planet as measured for De-rotate images (the Stack step): the ellipse of the globe, for Saturn the outer edge of the ring, and a
    /// short stroke with an "N" where the north pole of the planet points.
    var disc: DiscShape?

    var body: some View {
        Canvas { context, _ in
            let scale = transform.scale
            for point in points {
                let center = CGPoint(x: Double(point.x) * scale + transform.x, y: Double(point.y) * scale + transform.y)
                let r = 3.2 * scale
                context.fill(Path(ellipseIn: CGRect(x: center.x - r, y: center.y - r, width: 2 * r, height: 2 * r)), with: .color(SP.alignpoint))
            }
            for section in sections {
                let center = CGPoint(x: Double(section.point.x) * scale + transform.x, y: Double(section.point.y) * scale + transform.y)
                let r = 7 * scale
                context.stroke(
                    Path(ellipseIn: CGRect(x: center.x - r, y: center.y - r, width: 2 * r, height: 2 * r)),
                    with: .color(SP.alignpointRing), lineWidth: 1.4
                )
                // The shift is only a few pixels, so the stroke shows it enlarged to stay visible.
                if let shift = section.shift {
                    var stroke = Path()
                    stroke.move(to: center)
                    stroke.addLine(to: CGPoint(x: center.x + Double(shift.dx) * 4 * scale, y: center.y + Double(shift.dy) * 4 * scale))
                    context.stroke(stroke, with: .color(SP.alignpointShift), lineWidth: 1.4)
                }
            }
            for rect in [region, draft].compactMap({ $0 }) {
                let frame = CGRect(
                    x: Double(rect.x) * scale + transform.x, y: Double(rect.y) * scale + transform.y,
                    width: Double(rect.width) * scale, height: Double(rect.height) * scale
                )
                context.stroke(Path(frame), with: .color(SP.accent), style: StrokeStyle(lineWidth: 1.5, dash: [6, 5]))
            }
            if let disc {
                var layer = context
                drawDisc(disc, in: &layer)
            }
        }
        .allowsHitTesting(false)
        .accessibilityHidden(true)
    }

    private func drawDisc(_ disc: DiscShape, in context: inout GraphicsContext) {
        let scale = transform.scale
        let center = CGPoint(x: disc.x * scale + transform.x, y: disc.y * scale + transform.y)
        var rotated = context
        rotated.translateBy(x: center.x, y: center.y)
        rotated.rotate(by: .degrees(disc.angle))
        func ellipse(_ a: Double, _ b: Double) -> Path {
            Path(ellipseIn: CGRect(x: -a * scale, y: -b * scale, width: 2 * a * scale, height: 2 * b * scale))
        }
        rotated.stroke(ellipse(disc.semiMajor, disc.semiMinor), with: .color(SP.accent), style: StrokeStyle(lineWidth: 1.5))
        if disc.ring.count == 2 {
            rotated.stroke(ellipse(disc.ring[0], max(disc.ring[1], 0.5)), with: .color(SP.accent.opacity(0.7)), style: StrokeStyle(lineWidth: 1, dash: [5, 4]))
        }
        // The north pole of the planet: a stroke out of the globe, so that the orientation can be checked at a glance.
        let north = CGPoint(x: disc.northX, y: disc.northY)
        let pole = disc.semiMinor * scale
        var axis = Path()
        axis.move(to: CGPoint(x: center.x + north.x * pole, y: center.y + north.y * pole))
        axis.addLine(to: CGPoint(x: center.x + north.x * (pole + 14), y: center.y + north.y * (pole + 14)))
        context.stroke(axis, with: .color(SP.accent), style: StrokeStyle(lineWidth: 1.5, lineCap: .round))
        context.draw(
            Text("N").font(.sp(SPMetrics.fontSizeSmall, .semibold)).foregroundStyle(SP.accent),
            at: CGPoint(x: center.x + north.x * (pole + 24), y: center.y + north.y * (pole + 24))
        )
    }
}
