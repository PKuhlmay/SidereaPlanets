import Foundation
import SidereaPlanetsKit

// The pure parts of the step Save and of the project file: the values of Create AVI, the names of the dialogs, what is sent to the
// core and how a project file turns into settings and back. Same rules as app/src/logic/{exporting,projectfile,stack}.ts and
// saving.ts; the calls themselves are in AppModel+Save.swift.

/// *Image size* of "Create aligned Sequence" (`alignedSize` of the Windows settings).
nonisolated enum AlignedSize: String, CaseIterable, Identifiable, Sendable {
    case original, minimum, maximum, roi

    var id: String { rawValue }

    var label: String {
        switch self {
        case .original: "Original image size"
        case .minimum: "Minimum area"
        case .maximum: "Maximum area"
        case .roi: "Region of Interest"
        }
    }
}

/// *Method* of "Create integrated sequence": only the moving average exists (Compressed is locked, as in the Windows app).
nonisolated enum IntegratedMethod: String, Sendable {
    case moving, compressed
}

/// The codecs of the list *Codec* (`codecs` of settings.ts); the list has no effect, the video is always uncompressed.
nonisolated let aviCodecs = ["DIB", "cvid", "i420", "iyuv", "mrle", "msvc", "uyvy", "yuy2", "yvu9", "yvyu"]

/// What the Save step keeps between two clicks: the file type the last Save image used, the name it wrote and whether a write is under way.
nonisolated struct SaveState: Equatable, Sendable {
    /// `saveChoice` of the Windows app: the menu next to the button sets it, ⌘S reuses it.
    var format = ExportFormat.tiff16
    /// The file name of the last saved sharpened image: the result of the step Save.
    var savedName = ""
    /// A file is being written: the window is busy and the buttons are locked.
    var isWriting = false
}

/// Which video Create AVI writes.
nonisolated enum VideoKind: Sendable {
    case aligned, integrated

    /// The name of the run in the window title and the status bar.
    var name: String { self == .aligned ? "Save Aligned" : "Save Integrated" }
    /// Appended to the name of the recording in the file dialog.
    var suffix: String { self == .aligned ? "_aligned" : "_integrated" }
}

// MARK: Dialogs

/// What a save dialog is asked to show: the proposed name (without extension) and folder, and the file type it writes.
nonisolated struct SaveRequest: Equatable, Sendable {
    var name: String
    var directory: URL?
    /// Without the dot; `nil` lets any name through.
    var fileExtension: String?
}

/// "JUPITER.AVI" -> "JUPITER"; a name without extension stays as it is (`stem` of exporting.ts).
nonisolated func fileStem(_ fileName: String) -> String {
    let name = fileName as NSString
    let stem = name.deletingPathExtension
    return stem.isEmpty ? fileName : stem
}

// MARK: What is sent to the core

/// What Save Aligned and Save Integrated send: the request of Stack plus the options of Create AVI (`buildSequenceRequest`).
nonisolated func sequenceSettings(_ stack: StackOptions, align: AlignOptions, data: AlignData) -> SequenceSettings {
    SequenceSettings(
        stack: stackSettings(stack, align: align, data: data),
        framerate: wholeNumber(stack.framerate, minimum: 1),
        sortOnQuality: stack.sortOnQuality,
        averageFrames: UInt64(wholeNumber(stack.averageFrames, minimum: 1))
    )
}

/// True when the values leave the image as it is: Save image then writes the sharpened image unchanged (`isNeutral` of colour.ts).
nonisolated func isNeutral(_ colour: ColorSettings) -> Bool {
    func still(_ shift: ChannelShift) -> Bool { shift.x == 0 && shift.y == 0 }
    return still(colour.shifts.red) && still(colour.shifts.green) && still(colour.shifts.blue)
        && colour.red == 100 && colour.green == 100 && colour.blue == 100
        && colour.black == 0 && colour.white == 255 && colour.gamma == 1
        && colour.contrast == 100 && colour.brightness == 0
}

// MARK: Project file

/// A JSON value: the sections of the project file are free-form per step, the interface only passes text.
nonisolated enum JSON: Codable, Equatable, Sendable {
    case null
    case bool(Bool)
    case number(Double)
    case string(String)
    case array([JSON])
    case object([String: JSON])

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if container.decodeNil() {
            self = .null
        } else if let value = try? container.decode(Bool.self) {
            self = .bool(value)
        } else if let value = try? container.decode(Double.self) {
            self = .number(value)
        } else if let value = try? container.decode(String.self) {
            self = .string(value)
        } else if let value = try? container.decode([JSON].self) {
            self = .array(value)
        } else {
            self = .object(try container.decode([String: JSON].self))
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .null: try container.encodeNil()
        case .bool(let value): try container.encode(value)
        case .number(let value): try container.encode(value)
        case .string(let value): try container.encode(value)
        case .array(let value): try container.encode(value)
        case .object(let value): try container.encode(value)
        }
    }

    /// The JSON text the core stores in the section.
    var text: String {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        return (try? encoder.encode(self)).flatMap { String(data: $0, encoding: .utf8) } ?? "null"
    }

    init?(text: String) {
        guard let value = try? JSONDecoder().decode(JSON.self, from: Data(text.utf8)) else { return nil }
        self = value
    }
}

/// Reads the keys of one section the way `mergeKnown` of projectfile.ts does: a value counts only if the key is there and has the type
/// the setting has; everything else keeps its value. A hand-edited or older file therefore cannot put wrong types into the settings.
nonisolated struct SectionReader {
    let values: [String: JSON]

    init(_ section: JSON?) {
        if case .object(let values)? = section { self.values = values } else { values = [:] }
    }

    func bool(_ key: String, _ target: inout Bool) {
        if case .bool(let value)? = values[key] { target = value }
    }

    /// A finite number, rounded to a whole number (the fields are whole numbers).
    func int(_ key: String, _ target: inout Int) {
        if case .number(let value)? = values[key], value.isFinite, abs(value) < 1e9 { target = Int(value.rounded()) }
    }

    func string(_ key: String, _ target: inout String) {
        if case .string(let value)? = values[key] { target = value }
    }

    /// A string that names one of the cases; another string keeps the value.
    func choice<T: RawRepresentable>(_ key: String, _ target: inout T) where T.RawValue == String {
        if case .string(let value)? = values[key], let found = T(rawValue: value) { target = found }
    }
}

nonisolated extension Intensity {
    /// The names of `Intensity` in settings.ts.
    var jsonName: String {
        switch self {
        case .default: "default"
        case .area3x3: "area3x3"
        case .lowest: "lowest"
        }
    }

    init?(jsonName: String) {
        guard let found = [Intensity.default, .area3x3, .lowest].first(where: { $0.jsonName == jsonName }) else { return nil }
        self = found
    }
}

/// The section `align` (settings.align of the Windows app without the threshold, which belongs to the points of the current frame).
nonisolated func alignSection(_ options: AlignOptions, show: ShowOptions) -> JSON {
    .object([
        "showFullImage": .bool(show.fullImage),
        "showRoi": .bool(show.roi),
        "showAlignpoints": .bool(show.alignpoints),
        "showAligndata": .bool(show.aligndata),
        "showFramelist": .bool(show.framelist),
        "showRegistrationgraph": .bool(show.registrationgraph),
        "showPrefilter": .bool(show.prefilter),
        "minDistance": .number(Double(options.minDistance)),
        "edgeDistance": .number(Double(options.edgeDistance)),
        "intensity": .string(options.intensity.jsonName),
        "lo": .number(Double(options.lo)),
        "hi": .number(Double(options.hi)),
        "showAlignment": .bool(options.showAlignment),
        "boxSize": .number(Double(options.boxSize)),
        "maxMovement": .number(Double(options.maxMovement)),
        "limitMode": .string(options.limitMode.rawValue),
        "limitPercent": .number(Double(options.limitPercent)),
        "limitCount": .number(Double(options.limitCount)),
    ])
}

nonisolated func applyAlignSection(_ section: JSON?, to options: inout AlignOptions, show: inout ShowOptions) {
    let read = SectionReader(section)
    read.bool("showFullImage", &show.fullImage)
    read.bool("showRoi", &show.roi)
    read.bool("showAlignpoints", &show.alignpoints)
    read.bool("showAligndata", &show.aligndata)
    read.bool("showFramelist", &show.framelist)
    read.bool("showRegistrationgraph", &show.registrationgraph)
    read.bool("showPrefilter", &show.prefilter)
    read.int("minDistance", &options.minDistance)
    read.int("edgeDistance", &options.edgeDistance)
    if case .string(let name)? = read.values["intensity"], let intensity = Intensity(jsonName: name) { options.intensity = intensity }
    read.int("lo", &options.lo)
    read.int("hi", &options.hi)
    read.bool("showAlignment", &options.showAlignment)
    read.int("boxSize", &options.boxSize)
    read.int("maxMovement", &options.maxMovement)
    read.choice("limitMode", &options.limitMode)
    read.int("limitPercent", &options.limitPercent)
    read.int("limitCount", &options.limitCount)
}

/// The section `stack`: the settings of the Stack tab the Mac app has (the options the core does not read yet are not in the Mac app).
nonisolated func stackSection(_ options: StackOptions) -> JSON {
    .object([
        "showFullImage": .bool(options.showFullImage),
        "showAlignSections": .bool(options.showAlignSections),
        "showStackgraph": .bool(options.showStackgraph),
        "showStacklist": .bool(options.showStacklist),
        "nearestPoints": .number(Double(options.nearestPoints)),
        "normalize": .bool(options.normalize),
        "correctGeometry": .bool(options.correctGeometry),
        "edgePixels": .number(Double(options.edgePixels)),
        "codec": .string(options.codec),
        "compressQuality": .number(Double(options.compressQuality)),
        "framerate": .number(Double(options.framerate)),
        "alignedSize": .string(options.alignedSize.rawValue),
        "sortOnQuality": .bool(options.sortOnQuality),
        "usePrefilters": .bool(options.usePrefilters),
        "integratedMethod": .string(options.integratedMethod.rawValue),
        "averageFrames": .number(Double(options.averageFrames)),
    ])
}

nonisolated func applyStackSection(_ section: JSON?, to options: inout StackOptions) {
    let read = SectionReader(section)
    read.bool("showFullImage", &options.showFullImage)
    read.bool("showAlignSections", &options.showAlignSections)
    read.bool("showStackgraph", &options.showStackgraph)
    read.bool("showStacklist", &options.showStacklist)
    read.int("nearestPoints", &options.nearestPoints)
    read.bool("normalize", &options.normalize)
    read.bool("correctGeometry", &options.correctGeometry)
    read.int("edgePixels", &options.edgePixels)
    read.string("codec", &options.codec)
    read.int("compressQuality", &options.compressQuality)
    read.int("framerate", &options.framerate)
    read.choice("alignedSize", &options.alignedSize)
    read.bool("sortOnQuality", &options.sortOnQuality)
    read.bool("usePrefilters", &options.usePrefilters)
    read.choice("integratedMethod", &options.integratedMethod)
    read.int("averageFrames", &options.averageFrames)
}

/// The version of the layout this app writes (`FORMAT_VERSION` of crates/io/src/project.rs).
nonisolated let projectFormatVersion: UInt32 = 1

/// The project of the open recording: its files and the sections (docs/formats.md, "Projektdatei").
nonisolated func makeProject(recording: [URL], sections: [String: JSON]) -> Project {
    Project(formatVersion: projectFormatVersion, recording: recording.map(\.path), sections: sections.mapValues(\.text))
}

/// The sections of a loaded project; one that is not valid JSON is left out (the core already refuses a file that is not).
nonisolated func sections(of project: Project) -> [String: JSON] {
    project.sections.compactMapValues { JSON(text: $0) }
}
