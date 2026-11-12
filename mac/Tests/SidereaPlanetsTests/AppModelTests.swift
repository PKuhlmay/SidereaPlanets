import CoreGraphics
import Foundation
import Testing

@testable import SidereaPlanets

@MainActor
@Suite struct AppModelTests {
    let backend = FakeBackend()
    let suite = "AppModelTests-\(UUID().uuidString)"
    let defaults: UserDefaults
    let model: AppModel
    let jupiter = URL(fileURLWithPath: "/data/JUPITER.AVI")

    init() throws {
        defaults = try #require(UserDefaults(suiteName: suite))
        model = AppModel(backend: backend, defaults: defaults)
    }

    /// Waits until `condition` holds; the frame reads run on other threads.
    private func eventually(_ condition: () async -> Bool) async -> Bool {
        for _ in 0..<400 {
            if await condition() { return true }
            try? await Task.sleep(for: .milliseconds(5))
        }
        return false
    }

    @Test func startsEmptyWithOnlyOpenAvailable() {
        #expect(model.progress.step == .empty)
        #expect(model.viewStep == .open)
        #expect(model.stepStates[.open]?.status == .possible)
        #expect(model.stepStates[.alignpoints]?.status == .locked)
        #expect(model.stepStates[.alignpoints]?.hint == "Open a recording first")
        #expect(!model.showsFrameBar)
        #expect(model.image == nil)
    }

    @Test func openingARecordingMovesOnToAlignpoints() async {
        await model.open([jupiter])
        #expect(model.progress.step == .loaded)
        #expect(model.viewStep == .alignpoints)
        #expect(model.stepStates[.open]?.status == .done)
        #expect(model.stepStates[.alignpoints]?.status == .possible)
        #expect(model.stepStates[.align]?.status == .locked)
        #expect(model.stepStates[.align]?.hint == "Run Set Alignpoints first")
        #expect(model.result(of: .open) == "JUPITER.AVI · 10 frames")
        #expect(model.status.result == "Loaded 10 frames")
        #expect(model.windowTitle == "Siderea Planets — JUPITER.AVI")
        #expect(model.frameIndex == 0)
        #expect(model.image != nil)
        #expect(model.imageCaption == "Frame 1 of 10")
        #expect(model.showsFrameBar)
        #expect(model.viewport.imageSize == CGSize(width: 1, height: 1))  // frame 0 of the fake is one pixel wide
    }

    @Test func lockedStepsDoNotOpenButDoneAndPossibleOnesDo() async {
        await model.open([jupiter])
        model.show(.align)
        #expect(model.viewStep == .alignpoints)
        model.show(.open)
        #expect(model.viewStep == .open)
        #expect(!model.showsFrameBar)
        model.show(.alignpoints)
        #expect(model.viewStep == .alignpoints)
    }

    @Test func aFailedOpenKeepsWhatWasOpenAndShowsTheError() async {
        await model.open([jupiter])
        await backend.setOpenError("not a recording")
        await model.open([URL(fileURLWithPath: "/data/notes.txt")])
        #expect(model.status.result == "Error: not a recording")
        #expect(model.details?.fileName == "JUPITER.AVI")
        #expect(model.recent.entries == [["/data/JUPITER.AVI"]])
        #expect(!model.isOpening)
    }

    @Test func aFailedFirstOpenLeavesTheWindowEmpty() async {
        await backend.setOpenError("not a recording")
        await model.open([jupiter])
        #expect(model.progress.step == .empty)
        #expect(model.viewStep == .open)
        #expect(model.recent.entries.isEmpty)
    }

    @Test func openingAnotherRecordingStartsOver() async {
        await model.open([jupiter])
        model.showFrame(4)
        #expect(await eventually { model.frameIndex == 4 && model.image != nil })
        await backend.setRecording(FakeBackend.info(frames: 3))
        await model.open([URL(fileURLWithPath: "/data/CLAVIUS.AVI")])
        #expect(model.frameIndex == 0)
        #expect(model.frameCount == 3)
        #expect(model.viewStep == .alignpoints)
        #expect(model.status.result == "Loaded 3 frames")
        #expect(model.recent.entries.first == ["/data/CLAVIUS.AVI"])
    }

    @Test func recentFilesAreKeptNewestFirstAndSurviveARestart() async {
        await model.open([jupiter])
        await model.open([URL(fileURLWithPath: "/data/CLAVIUS.AVI")])
        await model.open([jupiter])
        #expect(model.recent.entries == [["/data/JUPITER.AVI"], ["/data/CLAVIUS.AVI"]])
        let restarted = AppModel(backend: backend, defaults: defaults)
        #expect(restarted.recent.entries == model.recent.entries)
        restarted.clearRecent()
        #expect(AppModel(backend: backend, defaults: defaults).recent.entries.isEmpty)
        defaults.removePersistentDomain(forName: suite)
    }

    @Test func frameIsClampedToTheRecording() async {
        await model.open([jupiter])
        model.showFrame(99)
        model.showFrame(-1)
        #expect(model.frameIndex == 0)
        model.showNextFrame(by: -1)
        #expect(model.frameIndex == 0)
    }

    @Test func fastSliderMovesReadOneFrameAtATimeAndSkipTheOnesInBetween() async {
        await model.open([jupiter])
        await backend.setHoldsReads(true)
        model.showFrame(3)
        #expect(await eventually { await backend.waitingReads == 1 })
        // Two more moves while frame 3 is still being read: only the newest one is read afterwards.
        model.showFrame(5)
        model.showFrame(7)
        #expect(model.frameIndex == 7)
        try? await Task.sleep(for: .milliseconds(50))
        #expect(await backend.requestedFrames == [0, 3])  // one read at a time
        await backend.releaseOneRead()
        #expect(await eventually { await backend.waitingReads == 1 })
        await backend.releaseOneRead()
        #expect(await eventually { await backend.requestedFrames.count == 3 })
        #expect(await backend.requestedFrames == [0, 3, 7])
        #expect(model.frameIndex == 7)
        #expect(model.image?.width == 8)
    }

    @Test func aFrameOfAReplacedRecordingIsDropped() async {
        await model.open([jupiter])
        await backend.setHoldsReads(true)
        model.showFrame(6)
        #expect(await eventually { await backend.waitingReads == 1 })
        // Opening the next recording meanwhile: its first frame must be the one on screen, not the late frame 6.
        await backend.setRecording(FakeBackend.info(frames: 3))
        let opening = Task { await model.open([URL(fileURLWithPath: "/data/CLAVIUS.AVI")]) }
        #expect(await eventually { await backend.waitingReads == 2 })
        await backend.releaseNewestRead()
        await opening.value
        // The late answer for frame 6 of the old recording arrives now and must not replace the new first frame.
        await backend.releaseOneRead()
        try? await Task.sleep(for: .milliseconds(50))
        #expect(model.frameIndex == 0)
        #expect(model.frameCount == 3)
        #expect(model.status.result == "Loaded 3 frames")
        #expect(model.image?.width == 1)
    }
}
