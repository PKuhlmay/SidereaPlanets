import CoreGraphics
import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// Stack in the app model, on the fake core: what is sent, how the steps move, what the status bar and the image area show, what Cancel
/// and Pause do, and that Set Alignpoints, Align and Limit take the stack back.
@MainActor
@Suite struct StackModelTests {
    let backend = FakeBackend()
    let defaults: UserDefaults
    let model: AppModel
    let jupiter = URL(fileURLWithPath: "/data/JUPITER.AVI")

    init() throws {
        defaults = try #require(UserDefaults(suiteName: "StackModelTests-\(UUID().uuidString)"))
        model = AppModel(backend: backend, defaults: defaults)
    }

    private func eventually(_ condition: () async -> Bool) async -> Bool {
        for _ in 0..<400 {
            if await condition() { return true }
            try? await Task.sleep(for: .milliseconds(5))
        }
        return false
    }

    /// A recording that went through Set Alignpoints, Align and Limit (frames 10, 6 and 3 kept, 6 points).
    private func limitedRecording() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        #expect(model.progress.step == .limited)
    }

    private func stackedRecording() async {
        await limitedRecording()
        await model.runStack()
        #expect(model.progress.step == .stacked)
    }

    // MARK: The run

    @Test func stackMakesTheImageOpensWaveletsAndReportsTheSize() async {
        await limitedRecording()
        #expect(model.viewStep == .stack)
        await model.runStack()

        #expect(model.progress.step == .stacked)
        #expect(model.viewStep == .wavelets)
        #expect(model.stepStates[.stack]?.status == .done)
        #expect(model.stepStates[.wavelets]?.status == .possible)
        #expect(model.stepStates[.save]?.status == .possible)
        #expect(model.stacked?.frames == 3 && model.stacked?.width == 1024 && model.stacked?.height == 680)
        #expect(await backend.stackImageCalls == 1)
        #expect(model.status.action == "Stack size (n=3) - (1024x680)")
        #expect(model.status.result == "Stacking Range : 0-255")
        #expect(model.result(of: .stack) == "3 frames · 1024×680")
        #expect(model.status.progress == 0 && model.run == nil && !model.isBusy)
    }

    @Test func stackNeedsLimit() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runStack()
        #expect(await backend.stackCalls.isEmpty)
        #expect(model.progress.step == .aligned && model.stacked == nil)
    }

    @Test func theRunShowsProgressTheTitleAndLocksTheWindowUntilItEnds() async {
        await backend.setKeptFrames([9, 5, 2, 0])
        await limitedRecording()
        await backend.setHoldsStack(true)
        let running = Task { await model.runStack() }
        #expect(await eventually { await backend.isStackHeld && model.status.progress == 0.5 })

        #expect(model.status.action == "Stacking frame: 2")
        #expect(model.windowTitle == "SidereaPlanets · Stack 50%")
        #expect(model.isBusy && model.canCancel && model.run?.operation == .stack)
        model.show(.limit)  // no pill opens during a calculation
        #expect(model.viewStep == .stack)
        await model.runStack()  // and no second run starts
        #expect(await backend.stackCalls.count == 1)

        await backend.finishStack()
        await running.value
        #expect(model.progress.step == .stacked && !model.isBusy && !model.canCancel)
        #expect(model.windowTitle == "SidereaPlanets — JUPITER.AVI")
    }

    @Test func cancelStopsTheRunAndLeavesTheStepReadyToRunAgain() async {
        await limitedRecording()
        await backend.setHoldsStack(true)
        let running = Task { await model.runStack() }
        #expect(await eventually { await backend.isStackHeld })

        model.cancelRun()
        await running.value

        #expect(backend.log.cancelledOperations == [.stack])
        #expect(model.status.result == "Cancelled")
        #expect(model.run == nil && !model.isBusy && !model.canCancel && model.status.progress == 0)
        #expect(model.progress.step == .limited && model.stacked == nil)
        #expect(model.viewStep == .stack && model.stepStates[.stack]?.status == .possible)
        #expect(await backend.stackImageCalls == 0)

        // The next run is not stopped by the old Cancel.
        await backend.setHoldsStack(false)
        await model.runStack()
        #expect(model.progress.step == .stacked)
    }

    @Test func aCancelledStackAgainKeepsTheEarlierStack() async {
        await stackedRecording()
        model.show(.stack)
        await backend.setHoldsStack(true)
        let running = Task { await model.runStack() }
        #expect(await eventually { await backend.isStackHeld })
        model.cancelRun()
        await running.value
        #expect(model.status.result == "Cancelled")
        #expect(model.progress.step == .stacked && model.stacked != nil)
        #expect(model.shownImage?.width == FakeBackend.stackPictureWidth)
    }

    @Test func pauseHoldsTheClockAndCancelResumesAPausedRun() async {
        await limitedRecording()
        await backend.setHoldsStack(true)
        let running = Task { await model.runStack() }
        #expect(await eventually { await backend.isStackHeld })

        model.togglePause()
        #expect(model.run?.paused == true)
        model.togglePause()
        #expect(model.run?.paused == false)
        model.togglePause()
        model.cancelRun()
        await running.value

        #expect(backend.log.pauseCalls == [true, false, true, false])  // Cancel resumes a paused run first
        #expect(backend.log.cancelledOperations == [.stack])
        #expect(model.status.result == "Cancelled")
    }

    @Test func aFailedStackShowsTheErrorAndStaysAtStack() async {
        await limitedRecording()
        await backend.setStackError("no alignpoints are checked")
        await model.runStack()
        #expect(model.status.result == "Error: no alignpoints are checked")
        #expect(model.progress.step == .limited && model.viewStep == .stack && model.stacked == nil)
        #expect(model.stepStates[.wavelets]?.status == .locked)
    }

    // MARK: What is sent

    @Test func theOptionsOfThePanelAndTheAlignTabGoToTheCore() async {
        await model.open([jupiter])
        model.options.boxSize = 24
        model.options.maxMovement = 7
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        model.stackOptions.nearestPoints = 5
        model.stackOptions.correctGeometry = false
        model.stackOptions.normalize = true
        model.stackOptions.edgePixels = 4
        await model.runStack()

        let sent = await backend.stackCalls[0]
        #expect(sent.nearestPoints == 5 && !sent.correctGeometry && sent.normalize && sent.edgeSkip == 4)
        #expect(sent.boxSize == 24 && sent.maxMovement == 7)
        #expect(sent.referenceFrame == 0 && sent.roi == nil)
        #expect(sent.frames == [2, 5, 9])  // the kept frames in frame order
        #expect(sent.points.count == 6 && sent.points[0] == PointPosition(x: 20, y: 10))
    }

    @Test func theDefaultsAreThoseOfTheWindowsApp() async {
        #expect(model.stackOptions == StackOptions(
            nearestPoints: 3, correctGeometry: true, normalize: false, edgePixels: 0,
            showFullImage: true, showAlignSections: true, showStackgraph: false, showStacklist: false
        ))
        await limitedRecording()
        await model.runStack()
        let sent = await backend.stackCalls[0]
        #expect(sent.nearestPoints == 3 && sent.correctGeometry && !sent.normalize && sent.edgeSkip == 0)
    }

    @Test func onlyCheckedPointsAndCheckedFramesAreStacked() async {
        await limitedRecording()
        model.setPointChecked(0, false)
        model.setPointChecked(3, false)
        model.setPointChecked(99, false)  // outside the list: ignored
        model.setFrameChecked(5, false)
        await model.runStack()

        let sent = await backend.stackCalls[0]
        #expect(sent.points.map(\.x) == [40, 60, 100, 120])
        #expect(sent.frames == [2, 9])
        #expect(model.stacked?.frames == 2)
    }

    @Test func theRegionOfInterestGoesToTheCoreToo() async {
        await model.open([jupiter])
        let roi = RoiSettings(x: 100, y: 50, width: 400, height: 300)
        model.selectRegion(roi)
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        await model.runStack()
        #expect(await backend.stackCalls[0].roi == roi)
    }

    // MARK: The image area

    @Test func theStackStepShowsTheFrameUntilTheStackExistsThenTheStack() async {
        await limitedRecording()
        #expect(model.viewStep == .stack && !model.showsStack)
        #expect(model.imageCaption == "Frame 1 of 10")
        #expect(model.shownImage?.width == 1)

        await model.runStack()
        #expect(model.viewStep == .wavelets)
        #expect(model.shownImage?.width == FakeBackend.stackPictureWidth)
        #expect(model.imageCaption == "Stack of 3 frames")
        model.show(.stack)
        #expect(model.shownImage?.width == FakeBackend.stackPictureWidth && model.imageCaption == "Stack of 3 frames")
        #expect(model.viewport.imageSize == CGSize(width: FakeBackend.stackPictureWidth, height: 1))

        // The steps before show the frame again.
        model.show(.limit)
        #expect(!model.showsStack && model.shownImage?.width == 1)
        #expect(model.imageCaption == "Frame 1 of 10")
        #expect(model.viewport.imageSize == CGSize(width: 1, height: 1))
    }

    @Test func theViewFitsTheImageWhenTheStepChanges() async {
        await stackedRecording()
        model.viewport.setBoxSize(CGSize(width: 800, height: 600))
        model.viewport.zoom(by: 2)
        #expect(!model.viewport.isFitted)
        model.show(.stack)
        #expect(model.viewport.isFitted)
    }

    // MARK: Taking the stack back

    @Test func setAlignpointsAgainTakesTheStackBack() async {
        await stackedRecording()
        model.show(.stack)
        await model.setAlignpoints()
        #expect(model.stacked == nil && model.progress.step == .points)
        #expect(model.viewStep == .align)
        #expect(model.stepStates[.stack]?.status == .locked && model.stepStates[.wavelets]?.status == .locked)
        #expect(model.result(of: .stack) == "")
    }

    @Test func alignAgainDropsTheStackAtOnce() async {
        await stackedRecording()
        await backend.setHoldsAlign(true)
        let running = Task { await model.runAlign() }
        #expect(await eventually { await backend.isAlignHeld })
        #expect(model.stacked == nil && model.progress.step == .points)  // gone when Align starts, not when it ends
        model.cancelRun()
        await running.value
        #expect(model.stacked == nil && model.stepStates[.stack]?.status == .locked)
    }

    @Test func limitAgainAfterAStackTakesItBackAndShowsTheFrame() async {
        await stackedRecording()
        model.show(.stack)
        #expect(model.showsStack)
        await model.runLimit()
        #expect(model.stacked == nil && model.progress.step == .limited)
        #expect(model.viewStep == .stack && !model.showsStack)
        #expect(model.imageCaption == "Frame 1 of 10" && model.shownImage?.width == 1)
        #expect(model.stepStates[.wavelets]?.status == .locked)
    }

    @Test func anotherRecordingTakesTheStackBack() async {
        await stackedRecording()
        await model.open([URL(fileURLWithPath: "/data/CLAVIUS.AVI")])
        #expect(model.stacked == nil && model.progress.step == .loaded)
        #expect(model.stackOptions.nearestPoints == 3)
    }

    // MARK: Alignmentpoints and the rings

    @Test func theRingsAppearAfterLimitWithTheShiftOfEachPoint() async {
        await backend.setPointShifts([PointShift(dx: 1, dy: -2), nil, PointShift(dx: 0.5, dy: 0.5)])
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()
        #expect(model.alignSections.isEmpty)  // not before Limit
        await model.runLimit()

        let sections = model.alignSections
        #expect(sections.count == 6)
        guard sections.count == 6 else { return }
        #expect(sections[0].shift == PointShift(dx: 1, dy: -2) && sections[1].shift == nil && sections[2].shift == PointShift(dx: 0.5, dy: 0.5))
        #expect(sections[3].shift == nil && sections[5].point == Point(x: 120, y: 60))
        model.stackOptions.showAlignSections = false
        #expect(model.alignSections.isEmpty)
    }

    @Test func aNewSetOfPointsChecksThemAllAndAlignDropsTheShifts() async {
        await backend.setPointShifts([PointShift(dx: 1, dy: 1)])
        await limitedRecording()
        model.setPointChecked(1, false)
        #expect(model.align.pointsChecked == [true, false, true, true, true, true])
        #expect(model.align.pointShifts.count == 1)
        await model.runAlign()
        #expect(model.align.pointShifts.isEmpty)
        await model.setAlignpoints()
        #expect(model.align.pointsChecked == [Bool](repeating: true, count: 6))
    }
}
