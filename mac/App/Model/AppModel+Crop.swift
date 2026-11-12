import CoreGraphics
import SidereaPlanetsKit

// The crop tool of the steps Wavelets and Save (MAC-016): a rectangle on the image that Save Image cuts out. It is opened by the Crop
// button of the Adjustments panel and by Crop… in the step Save, which is the same tool. Done keeps the rectangle (`crop.applied`) until
// the stack it belongs to is gone; Cancel throws away what was changed since the tool opened.

/// The name of the Crop button in the Adjustments panel.
nonisolated let cropFunction = "Crop"

extension AppModel {
    /// The size of the picture the rectangle is set in: the stack after Flip and Rotate, which is what the window shows (MAC-017);
    /// `nil` before the stack exists.
    var cropImageSize: (width: Int, height: Int)? { orientedSize }

    /// Crop is possible where the sharpened stack is shown (Wavelets and Save) and while nothing runs.
    var canCrop: Bool { stacked != nil && !isBusy && (viewStep == .wavelets || viewStep == .save) }

    /// The shape the rectangle keeps while it is dragged; `nil` while the tool is closed or the format is Free.
    var cropAspect: CropAspect? { crop.isEditing ? crop.format.aspect : nil }

    /// The rectangle Save Image cuts out, for the file it writes now: only the sharpened image (the Wavelets and Save steps, where the
    /// rectangle is set and shown) is cropped, the stack written from the Stack step is not.
    var saveCrop: CropRect? {
        guard saveSource == .wavelet, let applied = crop.applied, let size = cropImageSize,
              !applied.isFull(imageWidth: size.width, imageHeight: size.height) else { return nil }
        return applied.coreRect
    }

    /// "1200 × 900 px" of the rectangle that is edited, or else of the applied one; `nil` without one.
    var cropSizeText: String? { (crop.isEditing ? crop.draft : crop.applied)?.sizeText }

    /// Opens the tool on the applied rectangle, or on one in the middle of the image (80 % of each side, in the shape of the format).
    func startCrop() {
        guard canCrop, !crop.isEditing, let size = cropImageSize else { return }
        // The split view and the crop rectangle exclude each other; the button comes up and the split view starts again only when asked.
        if compareIsOn { closeFunction(compareName) }
        crop.draft = crop.applied
            ?? CropRectangle.centred(share: 0.8, imageWidth: size.width, imageHeight: size.height, aspect: crop.format.aspect)
        crop.isEditing = true
    }

    /// Cancel: closes the tool and drops what was changed since it opened; the applied rectangle stays as it was.
    func cancelCrop() {
        crop.isEditing = false
        crop.draft = nil
    }

    /// Done: the rectangle being edited is what Save Image cuts out. A rectangle that is the whole image crops nothing, so it is none.
    func finishCrop() {
        guard crop.isEditing else { return }
        if let draft = crop.draft, let size = cropImageSize, !draft.isFull(imageWidth: size.width, imageHeight: size.height) {
            crop.applied = draft
        } else {
            crop.applied = nil
        }
        cancelCrop()
    }

    /// Reset: the rectangle is the whole image again, in the format Free. The next drag on the image draws a new one.
    func resetCrop() {
        guard crop.isEditing, let size = cropImageSize else { return }
        crop.format = .free
        crop.draft = CropRectangle.full(imageWidth: size.width, imageHeight: size.height)
    }

    /// Takes the applied rectangle away: Save Image writes the whole image again.
    func removeCrop() {
        crop.applied = nil
    }

    /// Chooses the format; a rectangle that is already there is reshaped (same centre and area, see `CropRectangle.fitted`).
    func setCropFormat(_ format: CropFormat) {
        crop.format = format
        guard crop.isEditing, let aspect = format.aspect, let draft = crop.draft, let size = cropImageSize else { return }
        crop.draft = draft.fitted(to: aspect, imageWidth: size.width, imageHeight: size.height)
    }

    /// The rectangle the user dragged; it is kept inside the image.
    func setCropDraft(_ rectangle: CropRectangle) {
        guard crop.isEditing, let size = cropImageSize else { return }
        crop.draft = rectangle.clamped(imageWidth: size.width, imageHeight: size.height)
    }

    /// The stack the rectangle belongs to is gone (or another one was made).
    func clearCrop() {
        if crop != CropState() { crop = CropState() }
    }

    /// Another step closes the tool: the rectangle is only edited where the sharpened stack is shown.
    func cropViewChanged() {
        if viewStep != .wavelets && viewStep != .save { cancelCrop() }
    }
}
