import CoreGraphics
import Foundation
import SidereaPlanetsKit

// The step Stack of the window: what its button does and how the stack moves the step states. The Windows counterpart is
// app/src/stack.ts. The calculation runs in the Rust core; this only starts it and keeps the picture it answers. Set Alignpoints,
// Align and Limit throw the stack away in the core (`Session`), and `progress` makes the window show the same.

extension AppModel {
    /// Stacks the kept frames, with the checked alignpoints and the options of the panel, and shows the result in the image area.
    func runStack() async {
        guard progress.step >= .limited, !isBusy else { return }
        let settings = stackSettings(stackOptions, align: options, data: align)
        await runJob("Stack", operation: .stack) {
            status.action = "Stacking frame: 0"
            let answer = try await backend.stack(settings) { [weak self] update in
                Task { @MainActor in self?.jobProgress(update, label: "Stacking frame") }
            }
            let image = try await backend.stackImage()
            stacked = StackedImage(image: image, frames: Int(answer.frames), width: Int(answer.width), height: Int(answer.height))
            progress = StepProgress(step: .stacked)
            advance()
            let messages = stackMessages(frames: Int(answer.frames), width: Int(answer.width), height: Int(answer.height))
            status.action = messages.action
            return messages.result
        }
    }

    /// A check mark of the card *Alignmentpoints*: only checked points go into the stack.
    func setPointChecked(_ index: Int, _ checked: Bool) {
        guard align.pointsChecked.indices.contains(index) else { return }
        align.pointsChecked[index] = checked
    }

    /// The points with a ring in the Stack step (after Limit, with Show AlignSections): the position and the shift Limit measured.
    var alignSections: [AlignSection] {
        guard progress.step >= .limited, stackOptions.showAlignSections else { return [] }
        return align.points.enumerated().map { index, point in
            AlignSection(point: point, shift: align.pointShifts.indices.contains(index) ? align.pointShifts[index] : nil)
        }
    }
}

/// One ring of the Stack step: a point and how far it moved.
struct AlignSection: Equatable {
    var point: Point
    var shift: PointShift?
}
