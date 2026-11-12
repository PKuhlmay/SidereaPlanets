import CoreGraphics

/// Zoom and pan of the image area: a point of the image maps to the area as `screen = image * scale + (x, y)`, with the
/// origin at the top left of the area. Same maths as app/src/logic/viewport.ts.
nonisolated struct ViewTransform: Equatable, Sendable {
    static let margin: CGFloat = 12
    static let minScale: CGFloat = 0.02
    static let maxScale: CGFloat = 32

    var scale: CGFloat = 1
    var x: CGFloat = 0
    var y: CGFloat = 0

    private static func clamp(_ scale: CGFloat) -> CGFloat {
        min(maxScale, max(minScale, scale))
    }

    /// The whole image centred in the area with a margin around it.
    static func fit(box: CGSize, image: CGSize, margin: CGFloat = margin) -> ViewTransform {
        guard image.width > 0, image.height > 0 else { return ViewTransform() }
        let scale = clamp(min((box.width - 2 * margin) / image.width, (box.height - 2 * margin) / image.height))
        return ViewTransform(scale: scale, x: (box.width - image.width * scale) / 2, y: (box.height - image.height * scale) / 2)
    }

    /// Zooms by `factor`, keeping the image point under `point` in place.
    func zoomed(by factor: CGFloat, at point: CGPoint) -> ViewTransform {
        let scale = Self.clamp(self.scale * factor)
        let k = scale / self.scale
        return ViewTransform(scale: scale, x: point.x - (point.x - x) * k, y: point.y - (point.y - y) * k)
    }

    /// The image at 100 % of its pixels, around the centre of the area.
    func actualSize(in box: CGSize) -> ViewTransform {
        zoomed(by: 1 / scale, at: CGPoint(x: box.width / 2, y: box.height / 2))
    }

    /// The image moved by `delta` points.
    func panned(by delta: CGSize) -> ViewTransform {
        ViewTransform(scale: scale, x: x + delta.width, y: y + delta.height)
    }

    /// "106%": the scale in percent of the image pixels.
    var label: String { "\(Int((scale * 100).rounded()))%" }

    /// The area the image covers.
    func frame(for image: CGSize) -> CGRect {
        CGRect(x: x, y: y, width: image.width * scale, height: image.height * scale)
    }
}
