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
        #expect(makeWindowTitle(fileName: "", busy: nil) == "SidereaPlanets")
        #expect(makeWindowTitle(fileName: "JUPITER.AVI", busy: nil) == "SidereaPlanets — JUPITER.AVI")
        #expect(makeWindowTitle(fileName: "JUPITER.AVI", busy: (name: "Align", progress: 0.4)) == "SidereaPlanets · Align 40%")
        #expect(formatDuration(65) == "00:01:05")
        #expect(formatDuration(3725) == "01:02:05")
    }
}
