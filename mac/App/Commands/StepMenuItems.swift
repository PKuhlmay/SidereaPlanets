import SwiftUI

/// Run > Go to Step: the seven steps of the step bar. ⌥⌘1 to ⌥⌘7, because ⌘1 is Actual Size in the View menu.
struct StepMenuItems: View {
    let model: AppModel

    var body: some View {
        ForEach(ViewStep.allCases) { step in
            Button("\(step.number) \(step.label)") { model.show(step) }
                .keyboardShortcut(KeyEquivalent(Character("\(step.number)")), modifiers: [.command, .option])
                .disabled(model.isBusy || model.progress.state(of: step).status == .locked)
        }
    }
}
