import SidereaPlanetsKit
import AppKit
import SwiftUI

/// The picture itself: a layer-backed view that shows the `CGImage` as the contents of one layer and moves and scales that
/// layer for zoom and pan. The pixels are uploaded once per frame; zooming and panning only change the layer frame, so
/// they cost nothing on the main thread. It also reads the Mac gestures:
///
/// - pinch (trackpad) zooms around the fingers, a two-finger double tap toggles Fit and 100 %
/// - scrolling with ⌥ or ⌘ held (mouse wheel or trackpad) zooms around the pointer; plain scrolling moves the image
/// - dragging moves the image, or, while R.o.Interest waits for a region, draws a rectangle (the cross-hair cursor), or, while the crop
///   tool is open, moves, resizes or draws the crop rectangle (MAC-016)
struct ImageCanvas: NSViewRepresentable {
    let image: CGImage?
    /// View Compare: the image shown left of the divider, at the same zoom and position; `nil` outside it.
    var beforeImage: CGImage?
    /// The divider as a share (0...1) of the width of the area.
    var divider = compareDividerStart
    let transform: ViewTransform
    let viewport: ImageViewport
    /// The next drag draws a rectangle instead of moving the image.
    var isSelecting = false
    /// The rectangle while it is drawn, in image pixels; `nil` when the drag ends.
    var onDraft: (RoiSettings?) -> Void = { _ in }
    /// A finished rectangle in image pixels; a drag too small to be a region is dropped.
    var onSelect: (RoiSettings) -> Void = { _ in }
    /// The crop tool is open: a drag edits `crop` (the rectangle drawn by `CropOverlay`) instead of moving the image.
    var isCropping = false
    var crop: CropRectangle?
    /// The shape the rectangle keeps while it is dragged; `nil` is free.
    var cropAspect: CropAspect?
    var onCropChange: (CropRectangle) -> Void = { _ in }

    func makeNSView(context: Context) -> ImageCanvasView {
        ImageCanvasView(viewport: viewport)
    }

    func updateNSView(_ view: ImageCanvasView, context: Context) {
        view.onDraft = onDraft
        view.onSelect = onSelect
        view.isSelecting = isSelecting
        view.onCropChange = onCropChange
        view.cropAspect = cropAspect
        view.crop = crop
        view.isCropping = isCropping
        view.show(image: image, before: beforeImage, divider: divider, transform: transform)
    }
}

final class ImageCanvasView: NSView {
    private let viewport: ImageViewport
    private let imageLayer = CALayer()
    /// View Compare: a layer with the same frame as `imageLayer`, in a container that is cut off at the divider.
    private let beforeClip = CALayer()
    private let beforeLayer = CALayer()
    private var image: CGImage?
    private var beforeImage: CGImage?
    private var divider = compareDividerStart
    private var transform = ViewTransform()
    private var lastDragPoint: CGPoint?
    /// The corner where the drag of a region started, in image pixels.
    private var corner: CGPoint?
    private var draft: RoiSettings?
    var onDraft: (RoiSettings?) -> Void = { _ in }
    var onSelect: (RoiSettings) -> Void = { _ in }
    var isSelecting = false {
        didSet {
            guard isSelecting != oldValue else { return }
            window?.invalidateCursorRects(for: self)
        }
    }
    var isCropping = false {
        didSet {
            guard isCropping != oldValue else { return }
            window?.invalidateCursorRects(for: self)
        }
    }
    /// The rectangle being edited; a rectangle that covers the whole image counts as none drawn yet, so the next drag starts a new one.
    var crop: CropRectangle? {
        didSet {
            guard crop != oldValue else { return }
            window?.invalidateCursorRects(for: self)
        }
    }
    var cropAspect: CropAspect?
    var onCropChange: (CropRectangle) -> Void = { _ in }
    /// The crop drag in progress: what was grabbed (`nil` = a new rectangle), where, and the rectangle then.
    private var cropDrag: (handle: CropRectangle.Handle?, start: CGPoint, rect: CropRectangle?)?

    init(viewport: ImageViewport) {
        self.viewport = viewport
        super.init(frame: .zero)
        wantsLayer = true
        layer?.masksToBounds = true
        imageLayer.anchorPoint = .zero
        imageLayer.borderColor = NSColor.white.withAlphaComponent(0.07).cgColor
        imageLayer.borderWidth = 1
        imageLayer.minificationFilter = .trilinear
        imageLayer.magnificationFilter = .linear
        layer?.addSublayer(imageLayer)
        beforeClip.anchorPoint = .zero
        beforeClip.masksToBounds = true
        beforeClip.isHidden = true
        beforeLayer.anchorPoint = .zero
        beforeLayer.borderColor = imageLayer.borderColor
        beforeLayer.borderWidth = 1
        beforeLayer.minificationFilter = .trilinear
        beforeClip.addSublayer(beforeLayer)
        layer?.addSublayer(beforeClip)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) { fatalError("not used") }

    func show(image: CGImage?, before: CGImage? = nil, divider: Double = compareDividerStart, transform: ViewTransform) {
        if image !== self.image {
            self.image = image
            CATransaction.begin()
            CATransaction.setDisableActions(true)
            imageLayer.contents = image
            CATransaction.commit()
        }
        if before !== beforeImage {
            beforeImage = before
            CATransaction.begin()
            CATransaction.setDisableActions(true)
            beforeLayer.contents = before
            CATransaction.commit()
        }
        self.divider = divider
        if transform != self.transform, isCropping { window?.invalidateCursorRects(for: self) }
        self.transform = transform
        placeImage()
    }

    override func layout() {
        super.layout()
        placeImage()
    }

    override func resetCursorRects() {
        if isCropping {
            addCropCursorRects()
        } else {
            addCursorRect(bounds, cursor: isSelecting ? .crosshair : .openHand)
        }
    }

    // MARK: Crop

    /// The rectangle to grab, `nil` while it is the whole image (nothing drawn yet).
    private var editableCrop: CropRectangle? {
        guard let image, let crop, !crop.isFull(imageWidth: image.width, imageHeight: image.height) else { return nil }
        return crop
    }

    /// How far from an edge or corner the pointer still takes hold of it, in image pixels at the current zoom. `mouseDown` and the cursor
    /// rects both use it, so the cursor shows what a click does.
    private var cropGrabReach: Double { Double(CropOverlay.Metrics.grabDistance) / Double(max(transform.scale, 0.01)) }

    /// A rectangle of the image in the coordinates of the view (bottom left origin).
    private func viewRect(x: Double, y: Double, width: Double, height: Double) -> CGRect {
        CGRect(
            x: transform.x + x * transform.scale, y: bounds.height - (transform.y + (y + height) * transform.scale),
            width: width * transform.scale, height: height * transform.scale
        )
    }

    /// Cursor rects of the crop tool: a crosshair for drawing a new rectangle, the open hand inside, a resize cursor on each edge and
    /// corner (added last, so they win where they overlap).
    private func addCropCursorRects() {
        addCursorRect(bounds, cursor: .crosshair)
        guard let rect = editableCrop else { return }
        let reach = rect.effectiveTolerance(cropGrabReach)
        let x = Double(rect.minX), y = Double(rect.minY), w = Double(rect.width), h = Double(rect.height)
        addVisibleCursorRect(viewRect(x: x, y: y, width: w, height: h), cursor: .openHand)
        let bands: [(CGRect, CropRectangle.Handle)] = [
            (viewRect(x: x + reach, y: y - reach, width: w - 2 * reach, height: 2 * reach), .top),
            (viewRect(x: x + reach, y: y + h - reach, width: w - 2 * reach, height: 2 * reach), .bottom),
            (viewRect(x: x - reach, y: y + reach, width: 2 * reach, height: h - 2 * reach), .left),
            (viewRect(x: x + w - reach, y: y + reach, width: 2 * reach, height: h - 2 * reach), .right),
        ]
        let corners: [(CGRect, CropRectangle.Handle)] = [
            (viewRect(x: x - reach, y: y - reach, width: 2 * reach, height: 2 * reach), .topLeft),
            (viewRect(x: x + w - reach, y: y - reach, width: 2 * reach, height: 2 * reach), .topRight),
            (viewRect(x: x - reach, y: y + h - reach, width: 2 * reach, height: 2 * reach), .bottomLeft),
            (viewRect(x: x + w - reach, y: y + h - reach, width: 2 * reach, height: 2 * reach), .bottomRight),
        ]
        for (area, handle) in bands + corners { addVisibleCursorRect(area, cursor: Self.cursor(for: handle)) }
    }

    /// Adds a cursor rect for the part of `area` inside the view, or nothing when none of it is visible.
    private func addVisibleCursorRect(_ area: CGRect, cursor: NSCursor) {
        guard let visible = Self.visibleCursorArea(area, in: bounds) else { return }
        addCursorRect(visible, cursor: cursor)
    }

    /// The part of `area` inside `bounds`, `nil` when they do not overlap. AppKit throws on the null rect that `intersection` returns
    /// for disjoint rectangles (a crop edge zoomed or panned out of sight), which crashed the app.
    static func visibleCursorArea(_ area: CGRect, in bounds: CGRect) -> CGRect? {
        let visible = area.intersection(bounds)
        return visible.isNull || visible.isEmpty ? nil : visible
    }

    /// The resize cursors with a diagonal arrow need macOS 15; before that the corners show the same arrows as the edges.
    private static func cursor(for handle: CropRectangle.Handle) -> NSCursor {
        if #available(macOS 15, *) {
            switch handle {
            case .left, .right: return .columnResize
            case .top, .bottom: return .rowResize
            case .topLeft: return .frameResize(position: .topLeft, directions: .all)
            case .topRight: return .frameResize(position: .topRight, directions: .all)
            case .bottomLeft: return .frameResize(position: .bottomLeft, directions: .all)
            case .bottomRight: return .frameResize(position: .bottomRight, directions: .all)
            case .body: return .openHand
            }
        }
        switch handle {
        case .left, .right: return .resizeLeftRight
        case .top, .bottom: return .resizeUpDown
        case .topLeft, .topRight, .bottomLeft, .bottomRight: return .crosshair
        case .body: return .openHand
        }
    }

    override var acceptsFirstResponder: Bool { true }

    /// Puts the layer where the transform says. The transform counts from the top left, the layer from the bottom left.
    private func placeImage() {
        guard let image else {
            imageLayer.isHidden = true
            return
        }
        let size = CGSize(width: image.width, height: image.height)
        let frame = transform.frame(for: size)
        CATransaction.begin()
        CATransaction.setDisableActions(true)
        imageLayer.isHidden = false
        imageLayer.frame = CGRect(x: frame.minX, y: bounds.height - frame.maxY, width: frame.width, height: frame.height)
        // From 200 % on the pixels are shown as pixels, not smoothed (docs/design.md).
        imageLayer.magnificationFilter = transform.scale >= 2 ? .nearest : .linear
        imageLayer.contentsScale = window?.backingScaleFactor ?? 2
        placeBefore()
        CATransaction.commit()
    }

    /// The left half of View Compare: the same frame as the image, seen through a window that ends at the divider. Inside a transaction.
    private func placeBefore() {
        guard let beforeImage else {
            beforeClip.isHidden = true
            return
        }
        let frame = transform.frame(for: CGSize(width: beforeImage.width, height: beforeImage.height))
        beforeClip.isHidden = false
        beforeClip.frame = CGRect(x: 0, y: 0, width: bounds.width * divider, height: bounds.height)
        beforeLayer.frame = CGRect(x: frame.minX, y: bounds.height - frame.maxY, width: frame.width, height: frame.height)
        beforeLayer.magnificationFilter = transform.scale >= 2 ? .nearest : .linear
        beforeLayer.contentsScale = imageLayer.contentsScale
    }

    /// A point of an event in the coordinates of the transform: from the top left of the view.
    private func point(of event: NSEvent) -> CGPoint {
        let local = convert(event.locationInWindow, from: nil)
        return CGPoint(x: local.x, y: bounds.height - local.y)
    }

    // MARK: Gestures

    override func magnify(with event: NSEvent) {
        viewport.zoom(by: 1 + event.magnification, at: point(of: event))
    }

    override func smartMagnify(with event: NSEvent) {
        if viewport.isFitted {
            viewport.showActualSize(at: point(of: event))
        } else {
            viewport.fit()
        }
    }

    override func scrollWheel(with event: NSEvent) {
        let precise = event.hasPreciseScrollingDeltas
        if !event.modifierFlags.isDisjoint(with: [.option, .command]) {
            let delta = event.scrollingDeltaY != 0 ? event.scrollingDeltaY : event.scrollingDeltaX
            // About 1 % per point on a trackpad, 10 % per notch on a wheel.
            viewport.zoom(by: exp(delta * (precise ? 0.01 : 0.1)), at: point(of: event))
        } else {
            let scale: CGFloat = precise ? 1 : 10
            viewport.pan(by: CGSize(width: event.scrollingDeltaX * scale, height: event.scrollingDeltaY * scale))
        }
    }

    /// A point of the area in image pixels, also outside the image and between pixels.
    private func imagePosition(_ point: CGPoint) -> CGPoint {
        CGPoint(x: (point.x - transform.x) / transform.scale, y: (point.y - transform.y) / transform.scale)
    }

    override func mouseDown(with event: NSEvent) {
        if isCropping, image != nil {
            let position = imagePosition(point(of: event))
            if let rect = editableCrop, let handle = rect.handle(atX: position.x, y: position.y, tolerance: cropGrabReach) {
                cropDrag = (handle, position, rect)
            } else {
                cropDrag = (nil, position, nil)
            }
            return
        }
        if isSelecting, let image {
            let start = imagePosition(point(of: event))
            corner = start
            setDraft(rectBetween(start, start, image: CGSize(width: image.width, height: image.height)))
            return
        }
        lastDragPoint = point(of: event)
        NSCursor.closedHand.set()
    }

    private func setDraft(_ rect: RoiSettings?) {
        draft = rect
        onDraft(rect)
    }

    override func mouseDragged(with event: NSEvent) {
        if let drag = cropDrag, let image {
            let position = imagePosition(point(of: event))
            let width = image.width, height = image.height
            let updated: CropRectangle?
            if let handle = drag.handle, let rect = drag.rect {
                updated = rect.dragged(handle, byX: position.x - drag.start.x, y: position.y - drag.start.y, imageWidth: width, imageHeight: height, aspect: cropAspect)
            } else {
                updated = CropRectangle.drawn(fromX: drag.start.x, y: drag.start.y, toX: position.x, y: position.y, imageWidth: width, imageHeight: height, aspect: cropAspect)
            }
            if let updated, updated != crop { onCropChange(updated) }
            return
        }
        if let corner, let image {
            setDraft(rectBetween(corner, imagePosition(point(of: event)), image: CGSize(width: image.width, height: image.height)))
            return
        }
        guard let last = lastDragPoint else { return }
        let now = point(of: event)
        // A click without moving pans by zero, which does not count as moving the image: it keeps following the area.
        viewport.pan(by: CGSize(width: now.x - last.x, height: now.y - last.y))
        lastDragPoint = now
    }

    override func mouseUp(with event: NSEvent) {
        if cropDrag != nil {
            cropDrag = nil
            window?.invalidateCursorRects(for: self)
            return
        }
        if corner != nil {
            if let draft, isRegion(draft) { onSelect(draft) }
            corner = nil
            setDraft(nil)
            return
        }
        lastDragPoint = nil
        NSCursor.openHand.set()
    }
}
