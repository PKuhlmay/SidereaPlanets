import CoreGraphics
import Observation

/// Zoom and position of the image in the image area. While `isFitted` the image follows the size of the area; any zoom or
/// drag by the user turns that off until Fit (docs/design.md, "Bildbereich").
@MainActor @Observable
final class ImageViewport {
    private(set) var transform = ViewTransform()
    private(set) var isFitted = true
    private(set) var boxSize = CGSize.zero
    private(set) var imageSize = CGSize.zero

    /// A new image size (another recording) starts fitted; frames of the same size keep the view.
    func setImageSize(_ size: CGSize) {
        guard size != imageSize else { return }
        imageSize = size
        fit()
    }

    func setBoxSize(_ size: CGSize) {
        guard size != boxSize else { return }
        boxSize = size
        if isFitted { applyFit() }
    }

    /// Fit: the whole image in the area again. Also used when the step changes, because the area changes with the frame bar.
    func fit() {
        isFitted = true
        applyFit()
    }

    func zoom(by factor: CGFloat, at point: CGPoint) {
        guard hasImage else { return }
        transform = transform.zoomed(by: factor, at: point)
        isFitted = false
    }

    /// Zoom around the centre of the area (buttons, menu).
    func zoom(by factor: CGFloat) {
        zoom(by: factor, at: CGPoint(x: boxSize.width / 2, y: boxSize.height / 2))
    }

    /// 100 % of the image pixels; a click on the zoom label does this.
    func showActualSize(at point: CGPoint? = nil) {
        guard hasImage else { return }
        let anchor = point ?? CGPoint(x: boxSize.width / 2, y: boxSize.height / 2)
        transform = transform.zoomed(by: 1 / transform.scale, at: anchor)
        isFitted = false
    }

    func pan(by delta: CGSize) {
        guard hasImage, delta != .zero else { return }
        transform = transform.panned(by: delta)
        isFitted = false
    }

    private var hasImage: Bool { imageSize.width > 0 && imageSize.height > 0 }

    private func applyFit() {
        guard hasImage, boxSize.width > 0, boxSize.height > 0 else { return }
        transform = ViewTransform.fit(box: boxSize, image: imageSize)
    }
}
