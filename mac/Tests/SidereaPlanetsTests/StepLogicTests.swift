import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// The table "Zustände" of docs/design.md: which pill is done, possible or locked in which state.
@Suite struct StepLogicTests {
    private func statuses(_ progress: StepProgress) -> [StepStatus] {
        ViewStep.allCases.map { progress.state(of: $0).status }
    }

    @Test func emptyWindowOnlyAllowsOpen() {
        let progress = StepProgress(step: .empty)
        #expect(statuses(progress) == [.possible, .locked, .locked, .locked, .locked, .locked, .locked])
        #expect(progress.nextOpen == .open)
        #expect(progress.state(of: .alignpoints).hint == "Open a recording first")
    }

    @Test func loadedRecordingOpensAlignpoints() {
        let progress = StepProgress(step: .loaded)
        #expect(statuses(progress) == [.done, .possible, .locked, .locked, .locked, .locked, .locked])
        #expect(progress.nextOpen == .alignpoints)
        #expect(progress.state(of: .align).hint == "Run Set Alignpoints first")
    }

    @Test func eachResultUnlocksTheNextStep() {
        #expect(statuses(StepProgress(step: .points)) == [.done, .done, .possible, .locked, .locked, .locked, .locked])
        #expect(statuses(StepProgress(step: .aligned)) == [.done, .done, .done, .possible, .locked, .locked, .locked])
        #expect(statuses(StepProgress(step: .limited)) == [.done, .done, .done, .done, .possible, .locked, .locked])
    }

    @Test func saveIsPossibleRightAfterStack() {
        let progress = StepProgress(step: .stacked)
        #expect(statuses(progress) == [.done, .done, .done, .done, .done, .possible, .possible])
        #expect(progress.nextOpen == .wavelets)
    }

    @Test func waveletsAndSaveOnlyCountWithTheirResults() {
        let sharpened = StepProgress(step: .stacked, waveletsDone: true)
        #expect(statuses(sharpened).suffix(2) == [.done, .possible])
        #expect(sharpened.nextOpen == .save)
        let saved = StepProgress(step: .stacked, waveletsDone: true, saved: true)
        #expect(statuses(saved).allSatisfy { $0 == .done })
        #expect(saved.nextOpen == .save)
        // A saved flag without the Do All result does not count: Save loses its check with Wavelets.
        #expect(StepProgress(step: .stacked, waveletsDone: false, saved: true).state(of: .save).status == .possible)
    }

    @Test func runningAnEarlierStepAgainDiscardsLaterOnes() {
        let limited = StepProgress(step: .limited)
        #expect(limited.againDiscards(.align))
        #expect(!limited.againDiscards(.limit))  // nothing is built on Limit yet
        #expect(StepProgress(step: .stacked).againDiscards(.limit))  // the stack is
        #expect(!StepProgress(step: .loaded).againDiscards(.open))
        #expect(!StepProgress(step: .stacked, waveletsDone: true).againDiscards(.wavelets))
    }

    @Test func titleAndDuration() {
        #expect(makeWindowTitle(fileName: "", busy: nil) == "Siderea Planets")
        #expect(makeWindowTitle(fileName: "JUPITER.AVI", busy: nil) == "Siderea Planets — JUPITER.AVI")
        #expect(makeWindowTitle(fileName: "JUPITER.AVI", busy: (name: "Align", progress: 0.4)) == "Siderea Planets · Align 40%")
        #expect(formatDuration(65) == "00:01:05")
        #expect(formatDuration(3725) == "01:02:05")
    }

    /// Step 1 opens videos and series of images, so its button says "Select Recording…" (Philipp, 2026-11-12).
    @Test func openStepButtonSaysSelectRecording() {
        #expect(ViewStep.open.action == "Select Recording…")
    }

    /// Buttons are Title Case (HIG): the Save button reads "Save Image".
    @Test func saveButtonIsTitleCase() {
        #expect(ViewStep.save.action == "Save Image")
    }

    /// Every step explains itself in a tooltip, and the button of a done step is "‹Action› Again" (MAC-014).
    @Test func stepsExplainThemselvesAndHaveAgainLabels() {
        #expect(ViewStep.allCases.allSatisfy { !$0.explanation.isEmpty && !$0.actionExplanation.isEmpty })
        #expect(ViewStep.wavelets.againLabel == "Do All Again")
        #expect(ViewStep.open.againLabel == "Select Another Recording…")
        // Step 4 is called Best Frames, its button Keep Best Frames.
        #expect(ViewStep.limit.label == "Best Frames" && ViewStep.limit.action == "Keep Best Frames")
        #expect(StepProgress(step: .aligned).state(of: .stack).hint == "Run Keep Best Frames first")
    }

    @Test func failuresNameWhatFailed() {
        #expect(failureText("Align", PlanetsError.Failed(message: "disk full")) == "Align failed: disk full")
    }
}
