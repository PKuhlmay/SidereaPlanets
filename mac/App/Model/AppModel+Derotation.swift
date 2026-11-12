import Foundation
import SidereaPlanetsKit

// De-rotate images in the Stack panel (MAC-020): what the panel shows about it (span, rotation, the measured disc) and when that is
// worked out again. The calculation is in the Rust core (`Session::derotation_info`); stacking itself is `runStack`.

extension AppModel {
    /// The file has a time for every frame: the Start time field is not needed.
    var hasFrameTimestamps: Bool { details?.info.hasTimestamps ?? false }

    /// The suggested start time comes from the date of the file, so it is only a guess.
    var startTimeIsGuess: Bool { details?.info.startTimeFromFileDate ?? false }

    /// The line and hint under De-rotate images; nothing while the option is off or the numbers are not there yet.
    var derotationLine: (text: String, hint: String?, isProblem: Bool)? {
        guard stackOptions.derotate, let derotation else { return nil }
        return derotationSummary(derotation)
    }

    /// The measured disc for the overlay in the Stack step: only with the option on.
    var derotationDisc: DiscShape? { stackOptions.derotate ? derotation?.disc : nil }

    /// A change of the panel: whatever De-rotate images depends on is measured again.
    func derotationOptionsChanged(from old: StackOptions) {
        let new = stackOptions
        guard old.derotate != new.derotate || old.derotatePlanet != new.derotatePlanet || old.cameraAngle != new.cameraAngle
            || old.mirrored != new.mirrored || old.startTime != new.startTime
        else { return }
        refreshDerotation()
    }

    /// Works the span, the rotation and the disc out again for the kept frames, after a short pause so that typing in a field does not start
    /// a measurement per key. Needs the result of Limit; before that, or with the option off, there is nothing to show. In the Stack step the
    /// reference frame is the picture, because the disc was measured in it.
    func refreshDerotation() {
        derotationTask?.cancel()
        guard let settings = derotationSettings(stackOptions), progress.step >= .limited else {
            derotation = nil
            return
        }
        let reference = UInt32(clamping: align.referenceFrame)
        let frames = align.checkedFrames
        derotationTask = Task { [weak self] in
            try? await Task.sleep(for: .milliseconds(120))
            guard !Task.isCancelled, let self else { return }
            do {
                let info = try await backend.derotationInfo(settings, referenceFrame: reference, frames: frames)
                guard !Task.isCancelled else { return }
                derotation = info
                if info.disc != nil, viewStep == .stack, stacked == nil, frameIndex != Int(reference) { showFrame(Int(reference)) }
            } catch {
                guard !Task.isCancelled else { return }
                derotation = nil
                status.result = failureText("De-rotation", error)
            }
        }
    }

    /// The suggestions a newly opened recording gives: the start time from the file (SER header, else the date of the file) and the planet
    /// from the file name.
    func suggestDerotation(for info: RecordingInfo, fileName: String) {
        stackOptions.startTime = info.startTime.map { Date(timeIntervalSince1970: $0) }
        if let planet = derotationPlanetFromName(text: fileName) { stackOptions.derotatePlanet = planet }
    }
}
