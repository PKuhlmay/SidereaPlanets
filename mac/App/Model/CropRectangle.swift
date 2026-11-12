import Foundation

// Ported from Siderea (SidereaCore/CropRectangle.swift): the geometry of the crop rectangle, GUI-free and in whole image pixels.

/// A fixed shape for the crop rectangle: `long` : `short`, such as 3 : 2. Whether the rectangle lies or stands is not part of it: it keeps
/// the orientation it has (or the drag gives it), see `ratio(portrait:)`.
nonisolated struct CropAspect: Sendable, Equatable {
    let long: Int
    let short: Int

    /// Nil unless 0 < `short` ≤ `long`.
    init?(long: Int, short: Int) {
        guard short > 0, long >= short else { return nil }
        self.long = long
        self.short = short
    }

    static let square = CropAspect(long: 1, short: 1)!
    static let threeByTwo = CropAspect(long: 3, short: 2)!
    static let threeByFour = CropAspect(long: 4, short: 3)!

    /// Width ÷ height of a rectangle of this shape lying down (`portrait` false) or standing up.
    func ratio(portrait: Bool) -> Double {
        let landscape = Double(long) / Double(short)
        return portrait ? 1 / landscape : landscape
    }
}

/// The rectangle a user draws on an image to crop it, in whole image pixels, top-left origin like `ImageBuffer`: `minX`/`minY` are the
/// first kept column and row, `maxX`/`maxY` the first one past the rectangle. All the geometry of dragging (new rectangle, edge, corner, whole rectangle) lives here, so the view only turns
/// pointer positions into image pixels. Nothing here is ever outside the image or smaller than `minimumSize`.
nonisolated struct CropRectangle: Sendable, Equatable {
    /// The smallest side a rectangle can be dragged down to.
    static let minimumSize = 8

    private(set) var minX: Int
    private(set) var minY: Int
    private(set) var maxX: Int
    private(set) var maxY: Int

    var width: Int { maxX - minX }
    var height: Int { maxY - minY }

    /// The edges as given; the caller keeps them inside the image and apart by `minimumSize`.
    init(minX: Int, minY: Int, maxX: Int, maxY: Int) {
        self.minX = minX
        self.minY = minY
        self.maxX = maxX
        self.maxY = maxY
    }

    /// The rectangle that is the whole image.
    static func full(imageWidth: Int, imageHeight: Int) -> CropRectangle {
        CropRectangle(minX: 0, minY: 0, maxX: imageWidth, maxY: imageHeight)
    }

    /// A rectangle of `share` (0...1) of each side of the image, in the middle, reshaped to `aspect` when there is one.
    static func centred(share: Double, imageWidth: Int, imageHeight: Int, aspect: CropAspect? = nil) -> CropRectangle {
        let w = max(Int((Double(imageWidth) * share).rounded()), minimumSize), h = max(Int((Double(imageHeight) * share).rounded()), minimumSize)
        let x = clamp((imageWidth - w) / 2, 0, imageWidth - w), y = clamp((imageHeight - h) / 2, 0, imageHeight - h)
        let rect = CropRectangle(minX: x, minY: y, maxX: min(x + w, imageWidth), maxY: min(y + h, imageHeight))
        guard let aspect else { return rect }
        return rect.fitted(to: aspect, imageWidth: imageWidth, imageHeight: imageHeight)
    }

    /// This rectangle held inside an image of the given size: sides between `minimumSize` and the image, then slid inside.
    func clamped(imageWidth: Int, imageHeight: Int) -> CropRectangle {
        let w = Self.clamp(width, min(Self.minimumSize, imageWidth), imageWidth), h = Self.clamp(height, min(Self.minimumSize, imageHeight), imageHeight)
        let x = Self.clamp(minX, 0, imageWidth - w), y = Self.clamp(minY, 0, imageHeight - h)
        return CropRectangle(minX: x, minY: y, maxX: x + w, maxY: y + h)
    }

    /// Whether the rectangle is the whole image, so there is nothing to crop.
    func isFull(imageWidth: Int, imageHeight: Int) -> Bool {
        minX <= 0 && minY <= 0 && maxX >= imageWidth && maxY >= imageHeight
    }

    /// The rule-of-thirds guide lines of the rectangle, in image pixels: the two vertical lines' x positions and the two horizontal lines'
    /// y positions, at 1/3 and 2/3 of the width and height. Not rounded: the lines are a display aid, and a side that does not divide by
    /// three puts them between pixels.
    var thirdsLines: (xs: [Double], ys: [Double]) {
        let x = Double(minX), y = Double(minY), w = Double(width), h = Double(height)
        return ([x + w / 3, x + w * 2 / 3], [y + h / 3, y + h * 2 / 3])
    }

    /// The parts of a rectangle a pointer can take hold of.
    enum Handle: Sendable, CaseIterable {
        case topLeft, top, topRight, right, bottomRight, bottom, bottomLeft, left, body
    }

    /// The grab reach actually used for `requested` image pixels: capped at a third of the shorter side, so the middle of a small rectangle
    /// stays its body. `handle(atX:y:tolerance:)` and the front end's cursor rects both use it, so the cursor shows what a click does.
    func effectiveTolerance(_ requested: Double) -> Double {
        min(requested, Double(min(width, height)) / 3)
    }

    /// The largest share of the shorter side an arm of the centre cross may reach from the centre .
    static let centreCrossMaxArmShare = 0.25

    /// The arm length of the centre cross for `requested` image pixels: capped at `centreCrossMaxArmShare` of the shorter side, so a
    /// small rectangle keeps a short cross. Display only.
    func centreCrossArm(_ requested: Double) -> Double {
        min(requested, Double(min(width, height)) * Self.centreCrossMaxArmShare)
    }

    /// What lies under the image position (`x`, `y`): a corner, an edge (only along its own length, `tolerance` image pixels wide), the
    /// inside, or nothing. Corners win over edges and edges over the inside, so a small rectangle can always be grabbed and resized. The
    /// reach is capped at a third of the shorter side, so the middle of a small rectangle is still its body and can be dragged as a whole.
    func handle(atX x: Double, y: Double, tolerance requested: Double) -> Handle? {
        let tolerance = effectiveTolerance(requested)
        func near(_ a: Int, _ b: Double) -> Bool { abs(Double(a) - b) <= tolerance }
        let alongX = x >= Double(minX) - tolerance && x <= Double(maxX) + tolerance
        let alongY = y >= Double(minY) - tolerance && y <= Double(maxY) + tolerance
        if near(minX, x) && near(minY, y) { return .topLeft }
        if near(maxX, x) && near(minY, y) { return .topRight }
        if near(maxX, x) && near(maxY, y) { return .bottomRight }
        if near(minX, x) && near(maxY, y) { return .bottomLeft }
        if near(minY, y) && alongX { return .top }
        if near(maxY, y) && alongX { return .bottom }
        if near(minX, x) && alongY { return .left }
        if near(maxX, x) && alongY { return .right }
        if x > Double(minX) && x < Double(maxX) && y > Double(minY) && y < Double(maxY) { return .body }
        return nil
    }

    /// This rectangle with `handle` moved by (`dx`, `dy`) image pixels (rounded to whole pixels): an edge or corner moves only the edges
    /// it names, stopping at the image and `minimumSize` short of the opposite edge; the body moves as a whole and stays inside the image.
    func dragged(
        _ handle: Handle, byX dx: Double, y dy: Double, imageWidth: Int, imageHeight: Int, aspect: CropAspect? = nil
    ) -> CropRectangle {
        if let aspect, handle != .body {
            return resized(handle, byX: dx, y: dy, imageWidth: imageWidth, imageHeight: imageHeight, aspect: aspect)
        }
        let stepX = Int(dx.rounded()), stepY = Int(dy.rounded())
        var result = self
        if handle == .body {
            result.minX = Self.clamp(minX + stepX, 0, max(imageWidth - width, 0))
            result.minY = Self.clamp(minY + stepY, 0, max(imageHeight - height, 0))
            result.maxX = result.minX + width
            result.maxY = result.minY + height
            return result
        }
        switch handle {
        case .left, .topLeft, .bottomLeft: result.minX = Self.clamp(minX + stepX, 0, maxX - Self.minimumSize)
        case .right, .topRight, .bottomRight: result.maxX = Self.clamp(maxX + stepX, min(minX + Self.minimumSize, imageWidth), imageWidth)
        default: break
        }
        switch handle {
        case .top, .topLeft, .topRight: result.minY = Self.clamp(minY + stepY, 0, maxY - Self.minimumSize)
        case .bottom, .bottomLeft, .bottomRight: result.maxY = Self.clamp(maxY + stepY, min(minY + Self.minimumSize, imageHeight), imageHeight)
        default: break
        }
        return result
    }

    /// The rectangle between two image positions, whichever way it is dragged, kept inside the image; nil while it is still smaller than
    /// `minimumSize` on a side.
    static func drawn(
        fromX x0: Double, y y0: Double, toX x1: Double, y y1: Double, imageWidth: Int, imageHeight: Int, aspect: CropAspect? = nil
    ) -> CropRectangle? {
        if let aspect { return drawn(fromX: x0, y: y0, toX: x1, y: y1, imageWidth: imageWidth, imageHeight: imageHeight, fixing: aspect) }
        let ax = clamp(Int(x0.rounded()), 0, imageWidth), bx = clamp(Int(x1.rounded()), 0, imageWidth)
        let ay = clamp(Int(y0.rounded()), 0, imageHeight), by = clamp(Int(y1.rounded()), 0, imageHeight)
        let rect = CropRectangle(minX: min(ax, bx), minY: min(ay, by), maxX: max(ax, bx), maxY: max(ay, by))
        return rect.width >= minimumSize && rect.height >= minimumSize ? rect : nil
    }

    // MARK: Fixed aspect ratio

    /// Whether this rectangle stands up; a square or a lying one does not.
    private var isPortrait: Bool { height > width }

    /// This rectangle reshaped to `aspect`: the same area and the same centre, in the same orientation, so switching between formats
    /// neither shrinks it nor throws it around. It is cut down only where the image is too small for that area, grown where the area would
    /// leave a side below `minimumSize`, and slid inside the image where the centre would push it past an edge.
    func fitted(to aspect: CropAspect, imageWidth: Int, imageHeight: Int) -> CropRectangle {
        let ratio = aspect.ratio(portrait: isPortrait)
        // Both sides stay at least `minimumSize` (the shorter one exactly that, like `resized`), and only an image too small for it cuts deeper.
        let smallest = max(Double(Self.minimumSize), Double(Self.minimumSize) * ratio)
        let reshapedWidth = min(max((Double(width * height) * ratio).squareRoot(), smallest), Double(imageWidth), Double(imageHeight) * ratio)
        let w = max(Int(reshapedWidth.rounded()), 1), h = max(Int((reshapedWidth / ratio).rounded()), 1)
        let x = Self.clamp(Int((Double(minX + maxX) / 2 - Double(w) / 2).rounded()), 0, imageWidth - w)
        let y = Self.clamp(Int((Double(minY + maxY) / 2 - Double(h) / 2).rounded()), 0, imageHeight - h)
        return CropRectangle(minX: x, minY: y, maxX: x + w, maxY: y + h)
    }

    /// An edge or corner dragged with the shape fixed. A corner keeps the opposite corner and follows the pointer along whichever axis it
    /// has moved further; an edge keeps the opposite edge and grows the other side evenly around its middle. It stops at the image and at
    /// `minimumSize`, and the height is whole pixels rounded from the width, so the shape is exact to half a pixel.
    private func resized(_ handle: Handle, byX dx: Double, y dy: Double, imageWidth: Int, imageHeight: Int, aspect: CropAspect) -> CropRectangle {
        let ratio = aspect.ratio(portrait: isPortrait)
        let smallest = max(Double(Self.minimumSize), Double(Self.minimumSize) * ratio)
        let imageW = Double(imageWidth), imageH = Double(imageHeight)
        let movesLeft = [.left, .topLeft, .bottomLeft].contains(handle), movesRight = [.right, .topRight, .bottomRight].contains(handle)
        let movesTop = [.top, .topLeft, .topRight].contains(handle), movesBottom = [.bottom, .bottomLeft, .bottomRight].contains(handle)
        let isCorner = (movesLeft || movesRight) && (movesTop || movesBottom)

        if isCorner {
            // The corner that stays, and which way the rectangle grows from it.
            let anchorX = Double(movesLeft ? maxX : minX), anchorY = Double(movesTop ? maxY : minY)
            let signX: Double = movesLeft ? -1 : 1, signY: Double = movesTop ? -1 : 1
            let pointerX = Double(movesLeft ? minX : maxX) + dx, pointerY = Double(movesTop ? minY : maxY) + dy
            var w = max((pointerX - anchorX) * signX, (pointerY - anchorY) * signY * ratio, smallest)
            w = min(w, signX > 0 ? imageW - anchorX : anchorX, (signY > 0 ? imageH - anchorY : anchorY) * ratio)
            let wi = Int(w.rounded()), hi = Int((Double(wi) / ratio).rounded())
            return CropRectangle(
                minX: signX > 0 ? Int(anchorX) : Int(anchorX) - wi, minY: signY > 0 ? Int(anchorY) : Int(anchorY) - hi,
                maxX: signX > 0 ? Int(anchorX) + wi : Int(anchorX), maxY: signY > 0 ? Int(anchorY) + hi : Int(anchorY)
            )
        }
        if movesLeft || movesRight {
            let anchorX = Double(movesLeft ? maxX : minX), signX: Double = movesLeft ? -1 : 1
            let centreY = Double(minY + maxY) / 2
            var w = max((Double(movesLeft ? minX : maxX) + dx - anchorX) * signX, smallest)
            w = min(w, signX > 0 ? imageW - anchorX : anchorX, 2 * min(centreY, imageH - centreY) * ratio)
            let wi = Int(w.rounded()), hi = Int((Double(wi) / ratio).rounded())
            let y = Self.clamp(Int((centreY - Double(hi) / 2).rounded()), 0, imageHeight - hi)
            return CropRectangle(minX: signX > 0 ? Int(anchorX) : Int(anchorX) - wi, minY: y, maxX: signX > 0 ? Int(anchorX) + wi : Int(anchorX), maxY: y + hi)
        }
        let anchorY = Double(movesTop ? maxY : minY), signY: Double = movesTop ? -1 : 1
        let centreX = Double(minX + maxX) / 2
        var h = max((Double(movesTop ? minY : maxY) + dy - anchorY) * signY, smallest / ratio)
        h = min(h, signY > 0 ? imageH - anchorY : anchorY, 2 * min(centreX, imageW - centreX) / ratio)
        let hi = Int(h.rounded()), wi = Int((Double(hi) * ratio).rounded())
        let x = Self.clamp(Int((centreX - Double(wi) / 2).rounded()), 0, imageWidth - wi)
        return CropRectangle(minX: x, minY: signY > 0 ? Int(anchorY) : Int(anchorY) - hi, maxX: x + wi, maxY: signY > 0 ? Int(anchorY) + hi : Int(anchorY))
    }

    /// A new rectangle of a fixed shape: it starts at the first position and grows toward the second, lying or standing as the drag is
    /// wider or taller, as large as the drag needs in both directions and as the image allows.
    private static func drawn(
        fromX x0: Double, y y0: Double, toX x1: Double, y y1: Double, imageWidth: Int, imageHeight: Int, fixing aspect: CropAspect
    ) -> CropRectangle? {
        let startX = Double(clamp(Int(x0.rounded()), 0, imageWidth)), startY = Double(clamp(Int(y0.rounded()), 0, imageHeight))
        let endX = Double(clamp(Int(x1.rounded()), 0, imageWidth)), endY = Double(clamp(Int(y1.rounded()), 0, imageHeight))
        let acrossX = abs(endX - startX), acrossY = abs(endY - startY)
        let ratio = aspect.ratio(portrait: acrossY > acrossX)
        let growsRight = endX >= startX, growsDown = endY >= startY
        var w = max(acrossX, acrossY * ratio)
        w = min(w, growsRight ? Double(imageWidth) - startX : startX, (growsDown ? Double(imageHeight) - startY : startY) * ratio)
        let wi = Int(w.rounded()), hi = Int((Double(wi) / ratio).rounded())
        guard wi >= minimumSize, hi >= minimumSize else { return nil }
        let x = Int(startX), y = Int(startY)
        return CropRectangle(minX: growsRight ? x : x - wi, minY: growsDown ? y : y - hi, maxX: growsRight ? x + wi : x, maxY: growsDown ? y + hi : y)
    }

    private static func clamp(_ value: Int, _ low: Int, _ high: Int) -> Int { min(max(value, low), max(high, low)) }
}
