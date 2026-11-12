import CoreGraphics
import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// Alignpoints, Align and Limit in the app model, on the fake core: how the steps move, what the status bar says, what Cancel and
/// Pause do, and that a new Set Alignpoints or Align takes back what was built on the old one.
@MainActor
@Suite struct AlignModelTests {
    let backend = FakeBackend()
    let suite = "AlignModelTests-\(UUID().uuidString)"
    let defaults: UserDefaults
    let model: AppModel
    let jupiter = URL(fileURLWithPath: "/data/JUPITER.AVI")

    init() throws {
        defaults = try #require(UserDefaults(suiteName: suite))
        model = AppModel(backend: backend, defaults: defaults)
    }

    private func eventually(_ condition: () async -> Bool) async -> Bool {
        for _ in 0..<400 {
            if await condition() { return true }
            try? await Task.sleep(for: .milliseconds(5))
        }
        return false
    }

    private func statuses() -> [StepStatus] {
        ViewStep.allCases.map { model.stepStates[$0]?.status ?? .locked }
    }

    /// A recording that went through Set Alignpoints, Align and Limit.
    private func limitedRecording() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        #expect(model.progress.step == .limited)
    }

    // MARK: Set Alignpoints

    @Test func setAlignpointsMovesOnToAlign() async {
        await model.open([jupiter])
        model.showFrame(3)
        #expect(await eventually { model.image?.width == 4 })
        await model.setAlignpoints()

        #expect(model.progress.step == .points)
        #expect(model.viewStep == .align)
        #expect(statuses() == [.done, .done, .possible, .locked, .locked, .locked, .locked])
        #expect(model.align.points.count == 6)
        #expect(model.align.total == 8 && model.align.threshold == 2)
        #expect(model.align.referenceFrame == 3)  // the points belong to the frame that was shown
        #expect(await backend.pointCalls.map(\.frame) == [3])
        #expect(await backend.pointCalls[0].settings == pointSettings(AlignOptions(), roi: nil))
        #expect(model.status.result == "6 alignpoints set")
        #expect(model.result(of: .alignpoints) == "6 set")
        #expect(model.status.progress == 0 && model.run == nil && !model.isBusy)
    }

    @Test func noPointsKeepTheStepOpenAndSayWhatToTry() async {
        await model.open([jupiter])
        await backend.setNoPoints(true)
        await model.setAlignpoints()
        #expect(model.progress.step == .loaded)
        #expect(model.viewStep == .alignpoints)
        #expect(model.status.result == "No alignpoints found: try other Lo and Hi values")
        #expect(model.stepStates[.align]?.status == .locked)
    }

    @Test func aRegionTooSmallForTheEdgeDistanceIsRefused() async {
        await model.open([jupiter])
        model.selectRegion(RoiSettings(x: 10, y: 10, width: 40, height: 100))
        await model.setAlignpoints()
        #expect(model.status.result == "The region is too small for Min distance from edge")
        #expect(await backend.pointCalls.isEmpty)
        #expect(model.progress.step == .loaded)
    }

    @Test func theThresholdSliderDropsThePointsUntilAlignHasRun() async {
        await model.open([jupiter])
        model.setThreshold(3)  // nothing is set yet: ignored
        #expect(model.align.points.isEmpty && !model.canMoveThreshold)
        await model.setAlignpoints()
        #expect(model.canMoveThreshold)
        model.setThreshold(5)
        #expect(model.align.threshold == 5)
        #expect(model.align.points.count == 3)
        #expect(backend.log.threshold == 5)
        await model.runAlign()
        #expect(!model.canMoveThreshold)
        model.setThreshold(1)  // fixed once Align has run
        #expect(model.align.points.count == 3 && backend.log.threshold == 5)
    }

    // MARK: R.o.Interest

    @Test func theRegionIsDrawnDroppedAndPassedToSetAlignpointsAndAlign() async {
        await model.open([jupiter])
        #expect(model.canDrawRegion)
        model.toggleRegionOfInterest()
        #expect(model.isDrawingRegion)
        model.toggleRegionOfInterest()  // pressed again while waiting: nothing to draw
        #expect(!model.isDrawingRegion && model.align.roi == nil && !model.show.roi)

        let roi = RoiSettings(x: 100, y: 50, width: 400, height: 300)
        model.toggleRegionOfInterest()
        model.selectRegion(roi)
        #expect(!model.isDrawingRegion && model.align.roi == roi && model.show.roi)
        await model.setAlignpoints()
        #expect(await backend.pointCalls[0].settings.roi == roi)
        await model.runAlign()
        #expect(await backend.alignCalls[0].roi == roi)
        #expect(!model.canDrawRegion)  // the region only changes before Align
    }

    // MARK: Align

    @Test func alignRatesTheFramesAndOpensLimit() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()

        #expect(model.progress.step == .aligned)
        #expect(model.viewStep == .limit)
        #expect(statuses() == [.done, .done, .done, .possible, .locked, .locked, .locked])
        #expect(model.align.isRated)
        #expect(model.align.frames[9].quality == 100)  // percent of the best frame
        #expect(model.align.frames[0].quality ?? 0 > 92.0 && model.align.frames[0].quality ?? 0 < 92.2)
        #expect(model.align.listed.first?.index == 9)
        #expect(model.result(of: .align) == "best quality 100 %")
        #expect(model.status.action == "Loading frame: 10")
        #expect(model.status.result == "Alignment finished.")
        #expect(model.status.progress == 0 && model.run == nil)  // the bar is empty again after a run
        #expect(await backend.alignCalls == [AlignSettings(boxSize: 30, maxMovement: 5, roi: nil)])
    }

    @Test func theRunShowsProgressTheTitleAndLocksTheWindowUntilItEnds() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await backend.setHoldsAlign(true)
        let running = Task { await model.runAlign() }
        #expect(await eventually { await backend.isAlignHeld && model.status.progress == 0.5 })

        #expect(model.status.action == "Loading frame: 5")
        #expect(model.windowTitle == "Siderea Planets · Align 50%")
        #expect(model.isBusy && model.canCancel && model.run?.operation == .align)
        model.show(.open)  // no pill opens during a calculation
        #expect(model.viewStep == .align)
        await model.open([jupiter])  // nor does another recording
        #expect(model.align.points.count == 6)
        await model.runLimit()  // and no second run starts
        #expect(model.progress.step == .points)

        await backend.finishAlign()
        await running.value
        #expect(model.progress.step == .aligned)
        #expect(!model.isBusy && !model.canCancel)
        #expect(model.windowTitle == "Siderea Planets — JUPITER.AVI")
    }

    @Test func cancelStopsTheRunAndLeavesNothingRunning() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await backend.setHoldsAlign(true)
        let running = Task { await model.runAlign() }
        #expect(await eventually { await backend.isAlignHeld })

        model.cancelRun()
        await running.value

        #expect(backend.log.cancelledOperations == [.align])
        #expect(model.run == nil && !model.isBusy && !model.canCancel)
        #expect(model.status.result == "Cancelled")
        #expect(model.status.progress == 0)
        // The old rating went when Align started: a cancelled run leaves the step at Align, ready to run again.
        #expect(model.progress.step == .points)
        #expect(!model.align.isRated)
        #expect(model.stepStates[.align]?.status == .possible)
    }

    @Test func pauseHoldsTheClockAndCancelResumesAPausedRun() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await backend.setHoldsAlign(true)
        let running = Task { await model.runAlign() }
        #expect(await eventually { await backend.isAlignHeld })

        model.togglePause()
        #expect(model.run?.paused == true)
        model.togglePause()
        #expect(model.run?.paused == false)
        model.togglePause()
        model.cancelRun()  // a paused run waits for Resume, so Cancel resumes it first
        await running.value

        #expect(backend.log.pauseCalls == [true, false, true, false])
        #expect(backend.log.cancelledOperations == [.align])
        #expect(model.status.result == "Cancelled")
    }

    @Test func aFailedAlignShowsTheErrorAndStaysAtAlign() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await backend.setAlignError("no alignpoints")
        await model.runAlign()
        #expect(model.status.result == "Error: no alignpoints")
        #expect(model.progress.step == .points)
        #expect(model.run == nil && model.status.progress == 0)
    }

    @Test func cancelAndPauseDoNothingWithoutARun() {
        model.cancelRun()
        model.togglePause()
        #expect(backend.log.cancelledOperations.isEmpty && backend.log.pauseCalls.isEmpty)
        #expect(!model.canCancel)
    }

    // MARK: Limit

    @Test func limitKeepsTheFramesAndOpensStack() async {
        await limitedRecording()
        #expect(model.viewStep == .stack)
        #expect(statuses() == [.done, .done, .done, .done, .possible, .locked, .locked])  // Stack is open after Limit
        #expect(model.progress.nextOpen == .stack)
        #expect(await backend.limitCalls == [.lowestQuality(value: 80)])
        #expect(model.align.frames.filter(\.checked).map(\.index) == [2, 5, 9])
        #expect(model.result(of: .limit) == "3 of 10 kept")
        #expect(model.status.action == "Processing limited to 3 frames")
        #expect(model.status.result.hasPrefix("Stacksize: 3  Low quality: 97.30"))
    }

    @Test func limitSendsTheChosenModeAndValue() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()
        model.options.limitMode = .bestCount
        model.options.limitCount = 7
        await model.runLimit()
        model.options.limitMode = .bestPercent
        model.options.limitPercent = 30
        await model.runLimit()
        #expect(await backend.limitCalls == [.bestCount(value: 7), .bestPercent(value: 30)])
    }

    @Test func limitNeedsAnAlignment() async {
        await model.open([jupiter])
        await model.runLimit()
        await model.setAlignpoints()
        await model.runLimit()
        #expect(await backend.limitCalls.isEmpty)
        #expect(model.progress.step == .points)
    }

    // MARK: Running a step again

    @Test func setAlignpointsAgainTakesBackAlignAndLimit() async {
        await limitedRecording()
        #expect(model.progress.againDiscards(.alignpoints))
        await model.setAlignpoints()

        #expect(model.progress.step == .points)
        #expect(model.viewStep == .align)
        #expect(statuses() == [.done, .done, .possible, .locked, .locked, .locked, .locked])
        #expect(!model.align.isRated)
        #expect(model.align.frames.allSatisfy { $0.checked })
    }

    @Test func alignAgainDropsLimitAtOnceAndOpensLimitWhenDone() async {
        await limitedRecording()
        model.show(.align)
        await backend.setHoldsAlign(true)
        let running = Task { await model.runAlign() }
        #expect(await eventually { await backend.isAlignHeld })
        // The old Limit is gone while the new Align runs, not only when it ends.
        #expect(model.progress.step == .points)
        #expect(model.stepStates[.limit]?.status == .locked)
        #expect(model.align.frames.allSatisfy { $0.checked && $0.quality == nil })

        await backend.finishAlign()
        await running.value
        #expect(model.progress.step == .aligned)
        #expect(model.viewStep == .limit)
        #expect(model.align.frames.allSatisfy { $0.checked })
        #expect(model.stepStates[.stack]?.status == .locked)
    }

    @Test func aShownStepThatBecomesLockedMovesToTheNextOpenOne() async {
        await limitedRecording()
        model.show(.limit)
        await model.setAlignpoints()
        #expect(model.viewStep == .align)
    }

    @Test func limitAgainAfterAStackMakesTheStackObsolete() async {
        await limitedRecording()
        model.progress = StepProgress(step: .stacked, waveletsDone: true, saved: true)
        await model.runLimit()
        #expect(model.progress == StepProgress(step: .limited))
        #expect(model.stepStates[.wavelets]?.status == .locked)
    }

    // MARK: Go to best frame

    @Test func goToBestFrameShowsTheBestFrameAndOpensAlignpointsWithTheHint() async {
        await limitedRecording()
        model.show(.limit)
        #expect(model.canGoToBestFrame)
        #expect(model.frameIndex == 0)

        model.goToBestFrame()

        #expect(model.frameIndex == 9)  // frame 10 in the frame bar: the first of the ranking
        #expect(await eventually { model.image?.width == 10 })
        #expect(model.viewStep == .alignpoints)
        #expect(model.showsFrameBar)
        #expect(model.align.bestFrameHint == 9)
        #expect(bestFrameHint(frame: 9) == "Best frame: 10 — run Set Alignpoints and Align again to use it as reference.")
    }

    @Test func goToBestFrameKeepsTheResults() async {
        await limitedRecording()
        let before = model.align
        let pointCalls = await backend.pointCalls.count
        let alignCalls = await backend.alignCalls.count
        model.goToBestFrame()

        #expect(model.progress.step == .limited)
        #expect(model.align.frames == before.frames)
        #expect(model.align.points == before.points && model.align.pointShifts == before.pointShifts)
        #expect(model.align.referenceFrame == before.referenceFrame)
        #expect(model.result(of: .limit) == "3 of 10 kept")
        #expect(await backend.pointCalls.count == pointCalls)
        #expect(await backend.alignCalls.count == alignCalls)
    }

    @Test func goToBestFrameWaitsForAlign() async {
        model.goToBestFrame()  // no recording
        #expect(!model.canGoToBestFrame && model.align.bestFrameHint == nil)
        await model.open([jupiter])
        await model.setAlignpoints()
        #expect(!model.canGoToBestFrame)
        let step = model.viewStep
        model.goToBestFrame()  // points only, nothing rated
        #expect(model.frameIndex == 0 && model.viewStep == step && model.align.bestFrameHint == nil)
        await model.runAlign()
        #expect(model.canGoToBestFrame)
    }

    @Test func goToBestFrameIsLockedDuringARun() async {
        await limitedRecording()
        await backend.setHoldsAlign(true)
        let running = Task { await model.runAlign() }
        #expect(await eventually { await backend.isAlignHeld })
        #expect(!model.canGoToBestFrame)
        model.goToBestFrame()
        #expect(model.align.bestFrameHint == nil && model.frameIndex == 0)
        await backend.finishAlign()
        await running.value
    }

    @Test func theHintStaysUntilSetAlignpointsOrAlignRunsAgain() async {
        await limitedRecording()
        model.goToBestFrame()
        model.showFrame(2)  // the user moves on: the hint still names the best frame
        #expect(model.align.bestFrameHint == 9)

        await model.setAlignpoints()  // Set Alignpoints on the frame that is shown: the new reference, the old hint is obsolete
        #expect(model.align.bestFrameHint == nil)
        #expect(model.align.referenceFrame == 2)
        #expect(!model.align.isRated)  // only now the results go

        await model.runAlign()
        model.goToBestFrame()
        #expect(model.align.bestFrameHint == 9)
        await model.runAlign()
        #expect(model.align.bestFrameHint == nil)
    }

    @Test func anotherRecordingForgetsTheHint() async {
        await limitedRecording()
        model.goToBestFrame()
        await backend.setRecording(FakeBackend.info(frames: 3))
        await model.open([URL(fileURLWithPath: "/data/CLAVIUS.AVI")])
        #expect(model.align.bestFrameHint == nil)
    }

    // MARK: Opening, Framelist, Advanced

    @Test func anotherRecordingStartsTheAlignTabOver() async {
        await limitedRecording()
        await backend.setRecording(FakeBackend.info(frames: 3))
        await model.open([URL(fileURLWithPath: "/data/CLAVIUS.AVI")])
        #expect(model.align == AlignData(frameCount: 3))
        #expect(!model.isDrawingRegion)
        #expect(model.progress.step == .loaded)
    }

    @Test func theFramelistChecksAndEmptyListSaysItIsNotAvailable() async {
        await model.open([jupiter])
        model.setFrameChecked(2, false)
        #expect(model.align.keptCount == 9)
        model.checkAllFrames(false)
        #expect(model.align.keptCount == 0)
        model.checkAllFrames(true)
        #expect(model.align.keptCount == 10)
        model.setFrameChecked(99, false)  // outside the list: ignored
        #expect(model.align.keptCount == 10)
        model.emptyList()
        #expect(model.status.result == "Empty List: not available yet")
    }

    @Test func theStepsUpToStackAreBuilt() {
        #expect(ViewStep.allCases.filter(\.isBuilt) == [.open, .alignpoints, .align, .limit, .stack, .wavelets, .save])
    }

    @Test func anOpenAdvancedSectionIsKeptAcrossLaunches() {
        #expect(model.advancedOpen.isEmpty)
        model.toggleAdvanced(.align)
        model.toggleAdvanced(.alignpoints)
        model.toggleAdvanced(.alignpoints)
        #expect(model.advancedOpen == [.align])
        #expect(AppModel(backend: backend, defaults: defaults).advancedOpen == [.align])
        model.toggleAdvanced(.align)
        #expect(AppModel(backend: backend, defaults: defaults).advancedOpen.isEmpty)
        defaults.removePersistentDomain(forName: suite)
    }
}
