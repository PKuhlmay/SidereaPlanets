import CoreGraphics
import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// De-rotate images in the app model, on the fake core: what a new recording suggests, when the span, the rotation and the disc are asked for
/// again, what the panel and the image area get from the answer, and what Stack sends.
@MainActor
@Suite struct DerotationModelTests {
    let backend = FakeBackend()
    let defaults: UserDefaults
    let model: AppModel
    let jupiter = URL(fileURLWithPath: "/data/JUPITER.AVI")

    init() throws {
        defaults = try #require(UserDefaults(suiteName: "DerotationModelTests-\(UUID().uuidString)"))
        model = AppModel(backend: backend, defaults: defaults)
    }

    private func eventually(_ condition: () async -> Bool) async -> Bool {
        for _ in 0..<400 {
            if await condition() { return true }
            try? await Task.sleep(for: .milliseconds(5))
        }
        return false
    }

    /// Alignpoints set on frame 3, aligned and limited (frames 9, 5 and 2 kept); the Stack step is open.
    private func limitedRecording(from url: URL? = nil) async {
        await model.open([url ?? jupiter])
        model.showFrame(3)
        #expect(await eventually { model.image?.width == 4 })
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        #expect(model.progress.step == .limited && model.viewStep == .stack)
    }

    // MARK: Suggestions

    @Test func aNewRecordingSuggestsTheStartTimeAndThePlanet() async {
        var info = FakeBackend.info()
        info.startTime = 1_678_482_000
        info.startTimeFromFileDate = true
        await backend.setRecording(info)
        await model.open([URL(fileURLWithPath: "/data/SATURN_2023.ser")])
        #expect(model.stackOptions.startTime == Date(timeIntervalSince1970: 1_678_482_000))
        #expect(model.stackOptions.derotatePlanet == .saturn)
        #expect(model.startTimeIsGuess && !model.hasFrameTimestamps)
        #expect(!model.stackOptions.derotate, "the suggestions never turn the option on")
    }

    @Test func aFileThatNamesNoPlanetKeepsTheChoiceAndNoStartTimeLeavesTheFieldEmpty() async {
        model.stackOptions.derotatePlanet = .mars
        model.stackOptions.startTime = Date(timeIntervalSince1970: 7)
        await model.open([URL(fileURLWithPath: "/data/capture.avi")])
        #expect(model.stackOptions.derotatePlanet == .mars)
        #expect(model.stackOptions.startTime == nil, "the start time of the last recording does not belong to this one")
    }

    @Test func aFileWithTimeStampsHidesTheStartTime() async {
        var info = FakeBackend.info()
        info.hasTimestamps = true
        await backend.setRecording(info)
        await model.open([jupiter])
        #expect(model.hasFrameTimestamps)
    }

    // MARK: Asking the core

    @Test func nothingIsAskedWhileTheOptionIsOff() async {
        await limitedRecording()
        #expect(await backend.derotationCalls.isEmpty)
        #expect(model.derotation == nil && model.derotationLine == nil && model.derotationDisc == nil)
    }

    @Test func turningTheOptionOnAsksForTheKeptFramesAndTheReferenceFrame() async throws {
        await limitedRecording()
        model.stackOptions.derotatePlanet = .jupiter
        model.stackOptions.cameraAngle = 20
        model.stackOptions.derotate = true
        #expect(await eventually { await backend.derotationCalls.count == 1 })
        let call = try #require(await backend.derotationCalls.first)
        #expect(call.referenceFrame == 3 && call.frames == [2, 5, 9])
        #expect(call.settings.planet == .jupiter && call.settings.cameraAngleDeg == 20 && call.settings.useFileTimestamps)
        #expect(await eventually { model.derotationLine?.text == "Span 3 min 12 s · rotation 5.8°" })
        #expect(model.derotationLine?.hint == nil)
        #expect(model.derotationDisc == FakeBackend.discShape)
    }

    @Test func theReferenceFrameIsShownBecauseTheDiscWasMeasuredInIt() async {
        await limitedRecording()
        model.showFrame(7)
        #expect(await eventually { model.image?.width == 8 })
        model.stackOptions.derotate = true
        #expect(await eventually { model.image?.width == 4 }, "frame 4 (index 3) is the reference frame")
        #expect(model.frameIndex == 3)
    }

    @Test func everySettingThatMattersMeasuresAgain() async {
        await limitedRecording()
        model.stackOptions.derotate = true
        #expect(await eventually { await backend.derotationCalls.count == 1 })
        model.stackOptions.derotatePlanet = .mars
        #expect(await eventually { await backend.derotationCalls.count == 2 })
        model.stackOptions.mirrored = true
        #expect(await eventually { await backend.derotationCalls.count == 3 })
        model.stackOptions.startTime = Date(timeIntervalSince1970: 1_700_000_000)
        #expect(await eventually { await backend.derotationCalls.count == 4 })
        model.setFrameChecked(2, false)
        #expect(await eventually { await backend.derotationCalls.count == 5 })
        // Another field of the panel does not matter.
        model.stackOptions.nearestPoints = 9
        try? await Task.sleep(for: .milliseconds(300))
        #expect(await backend.derotationCalls.count == 5)
    }

    @Test func aFastSeriesOfChangesMeasuresOnce() async {
        await limitedRecording()
        model.stackOptions.derotate = true
        for angle in 1...6 { model.stackOptions.cameraAngle = Double(angle) }
        try? await Task.sleep(for: .milliseconds(500))
        let calls = await backend.derotationCalls
        #expect(calls.count == 1 && calls[0].settings.cameraAngleDeg == 6)
    }

    @Test func turningTheOptionOffTakesTheNumbersAndTheDiscAway() async {
        await limitedRecording()
        model.stackOptions.derotate = true
        #expect(await eventually { model.derotationDisc != nil })
        model.stackOptions.derotate = false
        #expect(model.derotation == nil && model.derotationDisc == nil && model.derotationLine == nil)
    }

    @Test func aShortRecordingGetsTheHint() async {
        await backend.setDerotation(DerotationInfo(
            spanSeconds: 40, rotationDegrees: 0.4, subEarthLatitude: 1, poleAngle: 2, disc: FakeBackend.discShape, problem: nil))
        await limitedRecording()
        model.stackOptions.derotate = true
        #expect(await eventually { model.derotationLine?.hint == "Too short to matter" })
        #expect(model.derotationLine?.text == "Span 40 s · rotation 0.4°")
    }

    @Test func aMissingStartTimeIsShownAsAProblemWithoutADisc() async {
        await backend.setDerotation(DerotationInfo(
            spanSeconds: 0, rotationDegrees: 0, subEarthLatitude: 0, poleAngle: 0, disc: nil,
            problem: "De-rotation needs the start time of the recording"))
        await limitedRecording()
        model.stackOptions.derotate = true
        #expect(await eventually { model.derotationLine?.isProblem == true })
        #expect(model.derotationDisc == nil)
    }

    @Test func nothingIsMeasuredBeforeLimit() async {
        await model.open([jupiter])
        model.stackOptions.derotate = true
        try? await Task.sleep(for: .milliseconds(300))
        #expect(await backend.derotationCalls.isEmpty && model.derotation == nil)
    }

    // MARK: Stack

    @Test func stackSendsDerotationWithTheSettingsOfThePanel() async throws {
        await limitedRecording()
        model.stackOptions.derotate = true
        model.stackOptions.derotatePlanet = .saturn
        model.stackOptions.startTime = Date(timeIntervalSince1970: 1_700_000_000)
        await model.runStack()
        let sent = try #require(await backend.stackCalls.first)
        #expect(sent.derotation == DerotationSettings(
            planet: .saturn, cameraAngleDeg: 0, mirrored: false, startTimeUtc: 1_700_000_000, useFileTimestamps: true))
    }

    @Test func stackWithTheOptionOffSendsNoDerotation() async throws {
        await limitedRecording()
        await model.runStack()
        #expect(try #require(await backend.stackCalls.first).derotation == nil)
    }

    @Test func aFailedStackShowsTheTextOfTheCore() async {
        await backend.setStackError("De-rotation needs the start time of the recording")
        await limitedRecording()
        model.stackOptions.derotate = true
        await model.runStack()
        #expect(model.status.result == "Stack failed: De-rotation needs the start time of the recording")
        #expect(model.progress.step == .limited)
    }

    // MARK: Project file

    @Test func theProjectKeepsTheSettingsInTheirOwnSection() async throws {
        model.stackOptions.derotate = true
        model.stackOptions.derotatePlanet = .saturn
        model.stackOptions.cameraAngle = 12.5
        model.stackOptions.startTime = Date(timeIntervalSince1970: 1_700_000_000)
        let sections = model.projectSections
        #expect(sections["derotation"] != nil)

        let other = AppModel(backend: FakeBackend(), defaults: try #require(UserDefaults(suiteName: "DerotationModelTests-\(UUID().uuidString)")))
        other.applyProjectSections(sections)
        #expect(other.stackOptions.derotate && other.stackOptions.derotatePlanet == .saturn && other.stackOptions.cameraAngle == 12.5)
        #expect(other.stackOptions.startTime == Date(timeIntervalSince1970: 1_700_000_000))
    }
}
