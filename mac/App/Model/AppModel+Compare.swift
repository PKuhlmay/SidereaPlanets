import CoreGraphics

// View Compare: the stack on the left of a divider and the image with sharpening and adjustments on the right, both at the same zoom and
// position. Only the view changes; nothing is calculated for it. The Before/After button of the zoom bar shows one of the two images
// in full; the two ways of looking never run at once.

extension AppModel {
    /// The View Compare button is on.
    var compareIsOn: Bool { wavelet.functions.contains(compareName) }

    /// The split view is shown: in the Wavelets step, with the stack as pixels, and not while Before/After shows the stack in full or the
    /// crop tool is open (the rectangle is set on one picture).
    var isComparing: Bool {
        viewStep == .wavelets && compareIsOn && waveletData.stack != nil && !waveletData.before && !crop.isEditing
    }

    /// The left half of the split view: the stack with nothing applied. The right half is `shownImage`.
    var compareBeforeImage: CGImage? { isComparing ? waveletData.stack?.image : nil }

    /// Moves the divider to a share (0...1) of the width of the image area; it stays within `compareDividerRange`.
    func setCompareDivider(_ share: Double) {
        let next = clampedDivider(share)
        if wavelet.compareDivider != next { wavelet.compareDivider = next }
    }

    /// The divider moved by the keyboard or VoiceOver.
    func nudgeCompareDivider(by delta: Double) {
        setCompareDivider(wavelet.compareDivider + delta)
    }

    /// The Before/After button: shows the stack in full, or the result again. Turning Before on ends the split view.
    func setBefore(_ on: Bool) {
        if on && compareIsOn { closeFunction(compareName) }
        waveletData.before = on
    }
}
