import SidereaPlanetsKit

/// What the Open panel and the step bar say about a loaded recording, with the labels of the Windows app.
nonisolated struct RecordingDetails: Equatable, Sendable {
    let fileName: String
    let info: RecordingInfo

    /// "DIB · 1024×680 · 10 frames": the line at the right of the step bar (after the file name).
    var summary: String {
        [info.codec, size, "\(info.frameCount) frames"].compactMap { $0 }.filter { !$0.isEmpty }.joined(separator: " · ")
    }

    /// The step bar line: file name first.
    var stepBarLine: String {
        ([fileName] + [summary]).filter { !$0.isEmpty }.joined(separator: " · ")
    }

    var size: String { "\(info.width)×\(info.height)" }

    /// The rows of the file info in the Open panel; observer, instrument and telescope only where the file has them.
    var rows: [(label: String, value: String)] {
        var rows: [(String, String)] = [("Format", info.format)]
        if let codec = info.codec, !codec.isEmpty { rows.append(("Codec", codec)) }
        rows.append(("Size", size))
        rows.append(("Bit depth", "\(info.bitDepth) bit"))
        rows.append(("Frames", "\(info.frameCount)"))
        rows.append(("Colour", info.color))
        if let fps = info.fps { rows.append(("Frame rate", Self.formatRate(fps))) }
        for (label, value) in [("Observer", info.observer), ("Instrument", info.instrument), ("Telescope", info.telescope)] {
            if let value, !value.isEmpty { rows.append((label, value)) }
        }
        return rows
    }

    /// "25 fps", "29.97 fps": up to two decimals, none for whole numbers.
    static func formatRate(_ fps: Double) -> String {
        let text = fps.rounded() == fps ? String(Int(fps)) : String(format: "%.2f", fps)
        return "\(text) fps"
    }
}
