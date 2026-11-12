import SwiftUI

/// The crop rectangle over the image (MAC-016, after Siderea's crop): everything outside is dimmed, the rectangle has a thin light border over
/// a dark one, a rule-of-thirds grid, a small cross at the centre, eight square grips and a "width × height" plate in its top left corner.
/// Without `isEditing` (the rectangle Done kept) only the dimmed outside, the border and the plate show. It lives in image pixels like the
/// other marks, so it zooms with the image, but every line is drawn in screen points. Never in the way of the mouse: the image view drags it.
/// The image area is dark in both appearances, so the colours are fixed (white over black), not the panel tokens.
struct CropOverlay: View {
    let transform: ViewTransform
    let rectangle: CropRectangle
    var isEditing = true

    /// Lengths in screen points, the values of Siderea's `Theme.Metrics.crop*`.
    enum Metrics {
        static let dimOpacity = 0.55
        static let appliedDimOpacity = 0.4
        static let borderWidth: CGFloat = 1
        static let borderUnderWidth: CGFloat = 3
        static let borderUnderOpacity = 0.6
        static let crossArm: CGFloat = 10
        static let gridWidth: CGFloat = 0.5
        static let gridUnderWidth: CGFloat = 2
        static let gridOpacity = 0.55
        static let gridUnderOpacity = 0.35
        static let gripSize: CGFloat = 8
        static let gripOutlineWidth: CGFloat = 1
        static let gripOutlineOpacity = 0.7
        static let plateOpacity = 0.6
        static let plateRadius: CGFloat = 3
        static let platePadding: CGFloat = 4
        static let plateInset: CGFloat = 10
        static let plateFontSize: CGFloat = 11
        /// How far from an edge or corner the pointer still takes hold of it.
        static let grabDistance: CGFloat = 7
    }

    var body: some View {
        Canvas { context, size in
            let scale = transform.scale
            let frame = CGRect(
                x: Double(rectangle.minX) * scale + transform.x, y: Double(rectangle.minY) * scale + transform.y,
                width: Double(rectangle.width) * scale, height: Double(rectangle.height) * scale
            )
            let shadow = Color.black, light = Color.white

            // Everything outside the rectangle, dimmed (even-odd: the rectangle is a hole).
            var dim = Path(CGRect(origin: .zero, size: size))
            dim.addRect(frame)
            context.fill(dim, with: .color(shadow.opacity(isEditing ? Metrics.dimOpacity : Metrics.appliedDimOpacity)), style: FillStyle(eoFill: true))

            // A dark line under a light one, so the border shows on bright and dark parts alike.
            let border = Path(frame)
            context.stroke(border, with: .color(shadow.opacity(Metrics.borderUnderOpacity)), lineWidth: Metrics.borderUnderWidth)
            context.stroke(border, with: .color(light), lineWidth: Metrics.borderWidth)

            if isEditing {
                var grid = Path()
                let thirds = rectangle.thirdsLines
                for x in thirds.xs {
                    let sx = x * scale + transform.x
                    grid.move(to: CGPoint(x: sx, y: frame.minY))
                    grid.addLine(to: CGPoint(x: sx, y: frame.maxY))
                }
                for y in thirds.ys {
                    let sy = y * scale + transform.y
                    grid.move(to: CGPoint(x: frame.minX, y: sy))
                    grid.addLine(to: CGPoint(x: frame.maxX, y: sy))
                }
                context.stroke(grid, with: .color(shadow.opacity(Metrics.gridUnderOpacity)), lineWidth: Metrics.gridUnderWidth)
                context.stroke(grid, with: .color(light.opacity(Metrics.gridOpacity)), lineWidth: Metrics.gridWidth)

                // The small cross at the centre, for lining an object up exactly; shorter on a small rectangle.
                let arm = rectangle.centreCrossArm(Double(Metrics.crossArm / scale)) * scale
                var cross = Path()
                cross.move(to: CGPoint(x: frame.midX - arm, y: frame.midY))
                cross.addLine(to: CGPoint(x: frame.midX + arm, y: frame.midY))
                cross.move(to: CGPoint(x: frame.midX, y: frame.midY - arm))
                cross.addLine(to: CGPoint(x: frame.midX, y: frame.midY + arm))
                context.stroke(cross, with: .color(shadow.opacity(Metrics.borderUnderOpacity)), style: StrokeStyle(lineWidth: Metrics.borderUnderWidth, lineCap: .square))
                context.stroke(cross, with: .color(light), style: StrokeStyle(lineWidth: Metrics.borderWidth, lineCap: .square))

                let grip = Metrics.gripSize
                let points = [
                    CGPoint(x: frame.minX, y: frame.minY), CGPoint(x: frame.midX, y: frame.minY), CGPoint(x: frame.maxX, y: frame.minY),
                    CGPoint(x: frame.minX, y: frame.midY), CGPoint(x: frame.maxX, y: frame.midY),
                    CGPoint(x: frame.minX, y: frame.maxY), CGPoint(x: frame.midX, y: frame.maxY), CGPoint(x: frame.maxX, y: frame.maxY),
                ]
                for point in points {
                    let square = Path(CGRect(x: point.x - grip / 2, y: point.y - grip / 2, width: grip, height: grip))
                    context.fill(square, with: .color(light))
                    context.stroke(square, with: .color(shadow.opacity(Metrics.gripOutlineOpacity)), lineWidth: Metrics.gripOutlineWidth)
                }
            }

            // The size, on a plate inside the top left corner.
            let label = context.resolve(
                Text(verbatim: "\(rectangle.width) × \(rectangle.height)")
                    .font(.system(size: Metrics.plateFontSize, weight: .medium).monospacedDigit())
                    .foregroundStyle(light)
            )
            let text = label.measure(in: CGSize(width: 400, height: 40))
            let plate = CGRect(
                x: frame.minX + Metrics.plateInset, y: frame.minY + Metrics.plateInset,
                width: text.width + 2 * Metrics.platePadding, height: text.height + Metrics.platePadding
            )
            context.fill(Path(roundedRect: plate, cornerRadius: Metrics.plateRadius), with: .color(shadow.opacity(Metrics.plateOpacity)))
            context.draw(label, at: CGPoint(x: plate.minX + Metrics.platePadding, y: plate.minY + Metrics.platePadding / 2), anchor: .topLeading)
        }
        .allowsHitTesting(false)
        .accessibilityHidden(true)
    }
}
