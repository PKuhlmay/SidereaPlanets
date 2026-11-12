import AppKit
import SwiftUI

/// The picture itself: a layer-backed view that shows the `CGImage` as the contents of one layer and moves and scales that
/// layer for zoom and pan. The pixels are uploaded once per frame; zooming and panning only change the layer frame, so
/// they cost nothing on the main thread. It also reads the Mac gestures:
///
/// - pinch (trackpad) zooms around the fingers, a two-finger double tap toggles Fit and 100 %
/// - scrolling with ⌥ or ⌘ held (mouse wheel or trackpad) zooms around the pointer; plain scrolling moves the image
/// - dragging moves the image
struct ImageCanvas: NSViewRepresentable {
    let image: CGImage?
    let transform: ViewTransform
    let viewport: ImageViewport

    func makeNSView(context: Context) -> ImageCanvasView {
        ImageCanvasView(viewport: viewport)
    }

    func updateNSView(_ view: ImageCanvasView, context: Context) {
        view.show(image: image, transform: transform)
    }
}

final class ImageCanvasView: NSView {
    private let viewport: ImageViewport
    private let imageLayer = CALayer()
    private var image: CGImage?
    private var transform = ViewTransform()
    private var lastDragPoint: CGPoint?

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
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) { fatalError("not used") }

    func show(image: CGImage?, transform: ViewTransform) {
        if image !== self.image {
            self.image = image
            CATransaction.begin()
            CATransaction.setDisableActions(true)
            imageLayer.contents = image
            CATransaction.commit()
        }
        self.transform = transform
        placeImage()
    }

    override func layout() {
        super.layout()
        placeImage()
    }

    override func resetCursorRects() {
        addCursorRect(bounds, cursor: .openHand)
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
        CATransaction.commit()
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

    override func mouseDown(with event: NSEvent) {
        lastDragPoint = point(of: event)
        NSCursor.closedHand.set()
    }

    override func mouseDragged(with event: NSEvent) {
        guard let last = lastDragPoint else { return }
        let now = point(of: event)
        // A click without moving pans by zero, which does not count as moving the image: it keeps following the area.
        viewport.pan(by: CGSize(width: now.x - last.x, height: now.y - last.y))
        lastDragPoint = now
    }

    override func mouseUp(with event: NSEvent) {
        lastDragPoint = nil
        NSCursor.openHand.set()
    }
}
