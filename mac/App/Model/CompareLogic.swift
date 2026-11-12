import CoreGraphics
import Foundation

// The pure parts of View Compare: the divider of the split view. It sits at a share of the width of the image area, so it stays where it
// is while the image is zoomed or moved, and both halves show the same part of the image.

/// The Adjustments button that switches the split view on and off.
nonisolated let compareName = "View Compare"

/// The divider keeps a sliver of each side, so that it can always be grabbed and neither image disappears.
nonisolated let compareDividerRange = 0.02...0.98

nonisolated let compareDividerStart = 0.5

nonisolated func clampedDivider(_ share: Double) -> Double {
    guard share.isFinite else { return compareDividerStart }
    return min(compareDividerRange.upperBound, max(compareDividerRange.lowerBound, share))
}

/// The divider for a pointer at `x` points in an area `width` points wide.
nonisolated func dividerShare(x: Double, width: Double) -> Double {
    width > 0 ? clampedDivider(x / width) : compareDividerStart
}

nonisolated enum CompareSide: Equatable, Sendable {
    case before, after
}

/// Whether the label of a side fits: it needs `needed` points between the divider and the edge of the area.
nonisolated func compareLabelFits(share: Double, width: Double, needed: Double, side: CompareSide) -> Bool {
    let room = side == .before ? share * width : (1 - share) * width
    return room >= needed
}
