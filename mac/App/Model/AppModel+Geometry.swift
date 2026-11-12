import CoreGraphics
import SidereaPlanetsKit

// The cards Flip and Rotate and Resize Image (MAC-017). Both are geometry steps at the end of Save Image, in the order refine, colors,
// flip and rotate, crop, resize. The picture the window shows in the steps Wavelets and Save is the flipped and rotated one, so the crop
// rectangle is set on exactly what is on screen; Resize Image does not change the preview, the cards tell the size that is written.

/// The name of the Flip and Rotate button in the Adjustments panel.
nonisolated let flipRotateFunction = "Flip and Rotate"
/// The name of the Resize Image button in the Adjustments panel.
nonisolated let resizeFunction = "Resize Image"

/// The images that are shown flipped and rotated: the one in the image area, and the left half of View Compare.
enum OrientedSlot: Hashable {
    case shown, before
}

/// A picture with the orientation applied, and what it was made from and with.
struct OrientedShot {
    let source: CGImage
    let orientation: OrientationSettings
    let shot: Shot
}

extension AppModel {
    // MARK: Sizes

    /// The size of the stack in pixels, before anything is flipped or turned.
    private var stackSize: (width: Int, height: Int)? {
        stacked.map { (width: $0.width, height: $0.height) }
    }

    /// The size of the picture after Flip and Rotate: the one the crop rectangle is set in. `nil` before the stack exists.
    var orientedSize: (width: Int, height: Int)? {
        guard let size = stackSize, !geometry.orientation.isNeutral else { return stackSize }
        let turned = orientationSize(width: UInt32(size.width), height: UInt32(size.height), orientation: geometry.orientation.core)
        return (Int(turned.width), Int(turned.height))
    }

    /// The size Resize Image starts from: the rectangle of the crop (the one being edited, or else the applied one), or the whole
    /// oriented picture.
    var resizeSource: (width: Int, height: Int)? { geometrySource(cropping: crop.isEditing ? crop.draft : crop.applied) }

    private func geometrySource(cropping rectangle: CropRectangle?) -> (width: Int, height: Int)? {
        guard let size = orientedSize else { return nil }
        if let rectangle, !rectangle.isFull(imageWidth: size.width, imageHeight: size.height) {
            return (width: rectangle.width, height: rectangle.height)
        }
        return size
    }

    /// The size of the file Save Image writes now (in the steps Wavelets and Save); `nil` before the stack exists.
    var resultSize: (width: Int, height: Int)? {
        guard let source = resizeSource else { return nil }
        return geometry.resize.size(source: source)
    }

    /// "1200 × 900 px".
    var resultSizeText: String? { resultSize.map { sizeText($0.width, $0.height) } }

    /// "680 × 1024 px": the picture after Flip and Rotate.
    var orientedSizeText: String? { orientedSize.map { sizeText($0.width, $0.height) } }

    // MARK: What Save Image does

    /// The geometry for the file Save Image writes now: only the sharpened image (the steps Wavelets and Save, where the cards are) gets
    /// it, like the crop; `nil` while it changes nothing.
    var saveGeometry: GeometrySettings? {
        guard saveSource == .wavelet, let source = geometrySource(cropping: crop.applied) else { return nil }
        let resize = geometry.resize.isNeutral(source: source) ? nil : geometry.resize.core
        if geometry.orientation.isNeutral && resize == nil { return nil }
        return GeometrySettings(orientation: geometry.orientation.core, resize: resize)
    }

    // MARK: Flip and Rotate

    /// The orientation cannot change while the crop tool is open: the rectangle is set on the picture as it is.
    var canEditOrientation: Bool { !crop.isEditing }

    /// The steps Wavelets and Save show the flipped and rotated picture.
    var orientationApplies: Bool {
        stacked != nil && !geometry.orientation.isNeutral && (viewStep == .wavelets || viewStep == .save)
    }

    private func editOrientation(_ change: (inout OrientationValues) -> Void) {
        guard canEditOrientation else {
            status.result = "Flip and Rotate: finish or cancel the crop first"
            return
        }
        var next = geometry.orientation
        change(&next)
        if next != geometry.orientation { geometry.orientation = next }
    }

    func flipHorizontally() { editOrientation { $0.flipHorizontally() } }
    func flipVertically() { editOrientation { $0.flipVertically() } }
    func rotate(by degrees: Double) { editOrientation { $0.rotate(by: degrees) } }
    func setAngle(_ degrees: Double) { editOrientation { $0.setAngle(degrees) } }
    func setCropToFit(_ on: Bool) { editOrientation { $0.cropToFit = on } }
    func setRotationMethod(_ method: RotationMethod) { editOrientation { $0.method = method } }
    /// Reset of the card: no flip, no turn.
    func resetOrientation() { editOrientation { $0 = OrientationValues() } }

    // MARK: Resize Image

    func setResizeMode(_ mode: ResizeMode) {
        guard let source = resizeSource else { return }
        if mode == .pixels { geometry.resize.switchToPixels(source: source) } else { geometry.resize.switchToPercent() }
    }

    func setResizePercent(_ percent: Int) {
        geometry.resize.percent = min(max(percent, ResizeLimits.percent.lowerBound), ResizeLimits.percent.upperBound)
    }

    func setResizeWidth(_ width: Int) {
        guard let source = resizeSource else { return }
        geometry.resize.setWidth(width, source: source)
    }

    func setResizeHeight(_ height: Int) {
        guard let source = resizeSource else { return }
        geometry.resize.setHeight(height, source: source)
    }

    func setResizeLock(_ on: Bool) {
        guard let source = resizeSource else { return }
        geometry.resize.setLock(on, source: source)
    }

    func setResizeMethod(_ method: ResizeMethod) { geometry.resize.method = method }

    /// Reset of the card: the picture keeps its size.
    func resetResize() { geometry.resize = ResizeValues() }

    // MARK: Hooks

    /// The values changed. A crop rectangle was set on the picture as it was before, so it is dropped when the picture was turned or
    /// flipped; the preview is made again.
    func geometryChanged(from old: GeometryValues) {
        if old.orientation.layout != geometry.orientation.layout, crop.applied != nil {
            crop.applied = nil
            status.result = "Crop removed: the picture was flipped or turned"
        }
        if old.orientation != geometry.orientation { requestOriented() }
    }

    // MARK: The flipped and rotated picture on screen

    /// The pictures that are shown flipped and rotated now, with what they start from.
    private var orientedTargets: [(slot: OrientedSlot, shot: Shot)] {
        guard orientationApplies else { return [] }
        var targets: [(slot: OrientedSlot, shot: Shot)] = []
        if let shot = waveletShownShot { targets.append((.shown, shot)) }
        if isComparing, let stack = waveletData.stack { targets.append((.before, stack)) }
        return targets
    }

    private func isOriented(_ entry: OrientedShot?, from shot: Shot) -> Bool {
        guard let entry else { return false }
        return entry.source === shot.image && entry.orientation == geometry.orientation.core
    }

    /// A shown picture has no flipped and rotated copy yet, or one made for other values.
    var orientedNeedsUpdate: Bool {
        orientedTargets.contains { !isOriented(waveletData.oriented[$0.slot], from: $0.shot) }
    }

    /// The picture for `slot`: the flipped and rotated copy once it is there (the previous one while the next is made, so the image does
    /// not jump), `plain` while there is none or nothing is flipped or turned.
    func orientedImage(_ plain: CGImage?, slot: OrientedSlot) -> CGImage? {
        guard orientationApplies else { return plain }
        return waveletData.oriented[slot]?.shot.image ?? plain
    }

    /// Makes the flipped and rotated copies: one request at a time, the newest values win.
    func requestOriented() {
        guard orientedNeedsUpdate else { return }
        let work = waveletWork
        work.orientAgain = true
        guard work.orientTask == nil else { return }
        work.orientTask = Task { [weak self] in
            guard let self else { return }
            while work.orientAgain {
                work.orientAgain = false
                await updateOriented()
            }
            work.orientTask = nil
        }
    }

    private func updateOriented() async {
        let orientation = geometry.orientation.core
        let version = waveletData.stackVersion
        for target in orientedTargets where !isOriented(waveletData.oriented[target.slot], from: target.shot) {
            do {
                let result = try await backend.orientPreview(target.shot.pixels, orientation)
                guard waveletData.stackVersion == version, let shot = Shot(result) else { return }
                waveletData.oriented[target.slot] = OrientedShot(source: target.shot.image, orientation: orientation, shot: shot)
            } catch {
                status.result = failureText("Flip and Rotate", error)
                return
            }
        }
        // The picture has another size now: the view fits it again.
        syncViewport()
        // Something changed while the core worked.
        if orientedNeedsUpdate { waveletWork.orientAgain = true }
    }
}
