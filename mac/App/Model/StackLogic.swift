import CoreGraphics
import Foundation
import SidereaPlanetsKit

// The pure parts of the step Stack: the values of the panel with the defaults of the Windows app, what is sent to the core and how
// its answer becomes what the window shows. Same rules as app/src/logic/stack.ts and stack.ts; the call itself is in
// AppModel+Stack.swift.

/// The values of the Stack panel (`settings.stack` of the Windows app) and the check marks of the Show menu in this step. The four options
/// at the top and De-rotate images (MAC-020) are read by the core; Output size other than Default, Stretch histogram, Resampling and
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
    // De-rotate images (MAC-020). Not in the Windows app's stack settings; the project file keeps them in their own section.
    var derotate = false
    var derotatePlanet = DerotatePlanet.jupiter
    /// Camera angle in degrees, counter-clockwise on the screen; 0 is celestial north up.
    var cameraAngle = 0.0
    /// The picture is a mirror image (east to the right with north up).
    var mirrored = false
    /// When the first frame was taken; used only when the file has no time stamps of its own.
    var startTime: Date?
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
        roi: data.roi,
        derotation: derotationSettings(stack)
    )
}

/// What the core needs for De-rotate images: `nil` when the option is off. The time stamps of the file are used whenever it has them.
nonisolated func derotationSettings(_ stack: StackOptions) -> DerotationSettings? {
    guard stack.derotate else { return nil }
    return DerotationSettings(
        planet: stack.derotatePlanet, cameraAngleDeg: stack.cameraAngle, mirrored: stack.mirrored,
        startTimeUtc: stack.startTime?.timeIntervalSince1970, useFileTimestamps: true
    )
}

/// A length of time as the panel shows it: "0.4 s", "45 s", "3 min 12 s", "1 h 05 min".
nonisolated func spanText(seconds: Double) -> String {
    if seconds < 10 { return String(format: "%.1f s", max(seconds, 0)) }
    let total = max(Int(seconds.rounded()), 0)
    if total < 60 { return "\(total) s" }
    if total < 3600 { return "\(total / 60) min \(total % 60) s" }
    return "\(total / 3600) h \(String(format: "%02d", total % 3600 / 60)) min"
}

/// The planet turns less than this many degrees in the recording: de-rotating changes next to nothing.
nonisolated let derotationNegligibleDegrees = 1.0

/// The line under De-rotate images: the span and how far the planet turns, and a hint when that is too little to matter; or what is missing.
nonisolated func derotationSummary(_ info: DerotationInfo) -> (text: String, hint: String?, isProblem: Bool) {
    if let problem = info.problem { return (problem, nil, true) }
    let text = "Span \(spanText(seconds: info.spanSeconds)) · rotation \(String(format: "%.1f", info.rotationDegrees))°"
    return (text, info.rotationDegrees < derotationNegligibleDegrees ? "Too short to matter" : nil, false)
}

/// The same time in UTC as "2026-11-12 08:38 UTC", for the line under the Start time field (the field itself shows local time).
nonisolated func utcText(_ date: Date) -> String {
    let formatter = DateFormatter()
    formatter.locale = Locale(identifier: "en_US_POSIX")
    formatter.timeZone = TimeZone(identifier: "UTC")
    formatter.dateFormat = "yyyy-MM-dd HH:mm 'UTC'"
    return formatter.string(from: date)
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

// MARK: Project file

private nonisolated func planetName(_ planet: DerotatePlanet) -> String {
    switch planet {
    case .jupiter: "jupiter"
    case .saturn: "saturn"
    case .mars: "mars"
    }
}

/// The section `derotation` of the project file, next to `align`, `stack` and the rest: De-rotate images with its planet, camera angle, mirror and
/// start time (seconds since 1970 UTC). Optional: written only by this app, and the Windows app, which does not know it, ignores it.
nonisolated func derotationSection(_ options: StackOptions) -> JSON {
    var object: [String: JSON] = [
        "enabled": .bool(options.derotate),
        "planet": .string(planetName(options.derotatePlanet)),
        "cameraAngle": .number(options.cameraAngle),
        "mirrored": .bool(options.mirrored),
    ]
    if let start = options.startTime { object["startTimeUtc"] = .number(start.timeIntervalSince1970) }
    return .object(object)
}

/// Reads the section back; a missing or wrong key keeps the value, so a project without the section leaves the options as they are.
nonisolated func applyDerotationSection(_ section: JSON?, to options: inout StackOptions) {
    let read = SectionReader(section)
    read.bool("enabled", &options.derotate)
    if case .string(let name)? = read.values["planet"] {
        switch name {
        case "jupiter": options.derotatePlanet = .jupiter
        case "saturn": options.derotatePlanet = .saturn
        case "mars": options.derotatePlanet = .mars
        default: break
        }
    }
    read.double("cameraAngle", &options.cameraAngle)
    options.cameraAngle = min(max(options.cameraAngle, -360), 360)
    read.bool("mirrored", &options.mirrored)
    if case .number(let seconds)? = read.values["startTimeUtc"], seconds.isFinite { options.startTime = Date(timeIntervalSince1970: seconds) }
}
