import CoreGraphics
import Foundation
import SidereaPlanetsKit

// The pure parts of the step Stack: the values of the panel with the defaults of the Windows app, what is sent to the core and how
// its answer becomes what the window shows. Same rules as app/src/logic/stack.ts and stack.ts; the call itself is in
// AppModel+Stack.swift.

/// The values of the Stack panel (`settings.stack` of the Windows app) and the check marks of the Show menu in this step. Only
/// the four options at the top are read by the core; Output size other than Default, Stretch histogram, De-rotate, Resampling and
/// Drizzling are locked in the panel (docs/design.md, "5 Stack").
nonisolated struct StackOptions: Equatable, Sendable {
    /// Use nearest points
    var nearestPoints = 3
    /// Correct geometry
    var correctGeometry = true
    /// Even out frame brightness
    var normalize = false
    /// Ignore edge pixels
    var edgePixels = 0
    // Create AVI (Advanced, MAC-009): Codec and Compression quality have no effect, the video is uncompressed; Frame size other than
    // the original and Use prefilters are not read yet either, so these controls are locked (docs/formats.md, "Create AVI"). The Windows app offers them all the same.
    var codec = "DIB"
    var compressQuality = 7500
    var framerate = 25
    var alignedSize = AlignedSize.original
    var sortOnQuality = false
    var usePrefilters = false
    var integratedMethod = IntegratedMethod.moving
    var averageFrames = 5
    // The Show menu of the Stack tab; Full Image, Stack Graph and Stack List have no function yet (dimmed in the menu, ShowMenuLogic.swift).
    var showFullImage = true
    var showAlignSections = true
    var showStackgraph = false
    var showStacklist = false
}

/// The stacked image as the Stack step shows it, and what the status bar and the result line say about it.
struct StackedImage {
    var image: CGImage
    /// How many frames went into the stack.
    var frames: Int
    var width: Int
    var height: Int
}

/// What the Stack button sends to the core: the checked alignpoints and frames together with the options of the panel and the
/// Align tab (Box size per point, Max. point shift).
nonisolated func stackSettings(_ stack: StackOptions, align: AlignOptions, data: AlignData) -> StackSettings {
    StackSettings(
        referenceFrame: UInt32(clamping: data.referenceFrame),
        points: data.usedPoints.map { PointPosition(x: $0.x, y: $0.y) },
        frames: data.checkedFrames,
        boxSize: wholeNumber(align.boxSize, minimum: 1),
        maxMovement: wholeNumber(align.maxMovement, minimum: 0),
        nearestPoints: UInt64(wholeNumber(stack.nearestPoints, minimum: 1)),
        correctGeometry: stack.correctGeometry,
        normalize: stack.normalize,
        edgeSkip: wholeNumber(stack.edgePixels, minimum: 0),
        roi: data.roi
    )
}

/// The status bar after Stack: the second field names the frames and the size ("Stacked 10 frames · 1024×680"), the third the range of
/// values (the stack always spans 0-255 here).
nonisolated func stackMessages(frames: Int, width: Int, height: Int) -> (action: String, result: String) {
    ("Stacked \(frames) \(frames == 1 ? "frame" : "frames") · \(width)×\(height)", "Value range 0–255")
}

/// The result line of the done step and the tooltip of its pill: "10 frames · 1024×680".
nonisolated func stackResultText(frames: Int, width: Int, height: Int) -> String {
    "\(frames) frames · \(width)×\(height)"
}
