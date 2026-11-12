import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// What Stack sends to the core and the texts it shows (same rules as app/src/logic/stack.ts).
@Suite struct StackLogicTests {
    private struct Missing: Error {}

    private func data(frames: Int = 5) -> AlignData {
        var data = AlignData(frameCount: frames)
        data.show(Alignpoints(points: [Point(x: 10, y: 20), Point(x: 30, y: 40), Point(x: 50, y: 60)], total: 3, threshold: 0))
        data.referenceFrame = 2
        return data
    }

    @Test func settingsCollectThePanelTheAlignTabAndTheChecks() {
        var data = data()
        data.frames[1].checked = false
        data.pointsChecked[1] = false
        data.roi = RoiSettings(x: 1, y: 2, width: 3, height: 4)
        var stack = StackOptions()
        stack.nearestPoints = 4
        stack.correctGeometry = false
        stack.normalize = true
        stack.edgePixels = 2
        var align = AlignOptions()
        align.boxSize = 40
        align.maxMovement = 9

        let sent = stackSettings(stack, align: align, data: data)
        #expect(sent.referenceFrame == 2)
        #expect(sent.points == [PointPosition(x: 10, y: 20), PointPosition(x: 50, y: 60)])
        #expect(sent.frames == [0, 2, 3, 4])
        #expect(sent.boxSize == 40 && sent.maxMovement == 9)
        #expect(sent.nearestPoints == 4 && !sent.correctGeometry && sent.normalize && sent.edgeSkip == 2)
        #expect(sent.roi == RoiSettings(x: 1, y: 2, width: 3, height: 4))
    }

    @Test func typedValuesAreHeldToWhatTheCoreCanTake() {
        var stack = StackOptions()
        stack.nearestPoints = 0  // at least one point decides a pixel
        stack.edgePixels = -3
        var align = AlignOptions()
        align.boxSize = 0
        align.maxMovement = -1
        let sent = stackSettings(stack, align: align, data: data())
        #expect(sent.nearestPoints == 1 && sent.edgeSkip == 0 && sent.boxSize == 1 && sent.maxMovement == 0)
    }

    @Test func textsAreThoseOfTheWindowsApp() {
        let messages = stackMessages(frames: 10, width: 1024, height: 680)
        #expect(messages.action == "Stacked 10 frames · 1024×680")
        #expect(messages.result == "Value range 0–255")
        #expect(stackResultText(frames: 8, width: 1024, height: 680) == "8 frames · 1024×680")
    }

    @Test func aNewSetOfPointsIsCheckedAndHasNoShifts() {
        var data = data()
        #expect(data.pointsChecked == [true, true, true] && data.pointShifts.isEmpty)
        data.pointShifts = [PointShift(dx: 1, dy: 2)]
        data.discardRating()
        #expect(data.pointShifts.isEmpty)
    }

    // MARK: Align by center of gravity (MAC-021)

    @Test func theStackRegistersWithTheCentreOfGravityOfTheAlignStep() {
        var align = AlignOptions()
        #expect(stackSettings(StackOptions(), align: align, data: data()).centreOfGravity == nil)
        align.centreOfGravity = true
        align.brightnessThreshold = 25
        #expect(stackSettings(StackOptions(), align: align, data: data()).centreOfGravity == 25)
    }

    // MARK: De-rotate images (MAC-020)

    @Test func derotationIsOffUnlessTheOptionIsOn() {
        #expect(stackSettings(StackOptions(), align: AlignOptions(), data: data()).derotation == nil)
        #expect(derotationSettings(StackOptions()) == nil)
    }

    @Test func derotationSendsPlanetAngleMirrorAndStartTime() {
        var stack = StackOptions()
        stack.derotate = true
        stack.derotatePlanet = .saturn
        stack.cameraAngle = -12.5
        stack.mirrored = true
        stack.startTime = Date(timeIntervalSince1970: 1_700_000_000.5)
        let sent = stackSettings(stack, align: AlignOptions(), data: data()).derotation
        #expect(sent == DerotationSettings(
            planet: .saturn, cameraAngleDeg: -12.5, mirrored: true, startTimeUtc: 1_700_000_000.5, useFileTimestamps: true))
        stack.startTime = nil
        #expect(derotationSettings(stack)?.startTimeUtc == nil)
    }

    @Test func aSpanIsWrittenInTheLargestUnitsThatFit() {
        #expect(spanText(seconds: 0) == "0.0 s")
        #expect(spanText(seconds: 0.36) == "0.4 s")
        #expect(spanText(seconds: 9.94) == "9.9 s")
        #expect(spanText(seconds: 44.6) == "45 s")
        #expect(spanText(seconds: 192) == "3 min 12 s")
        #expect(spanText(seconds: 3600) == "1 h 00 min")
        #expect(spanText(seconds: 3900) == "1 h 05 min")
        #expect(spanText(seconds: -4) == "0.0 s")
    }

    private func info(span: Double, rotation: Double, problem: String? = nil) -> DerotationInfo {
        DerotationInfo(
            spanSeconds: span, rotationDegrees: rotation, subEarthLatitude: 0, poleAngle: 0, disc: nil, problem: problem)
    }

    @Test func theSummaryNamesSpanAndRotationAndWarnsWhenItIsTooLittle() {
        let normal = derotationSummary(info(span: 192, rotation: 5.84))
        #expect(normal.text == "Span 3 min 12 s · rotation 5.8°" && normal.hint == nil && !normal.isProblem)
        let short = derotationSummary(info(span: 90, rotation: 0.91))
        #expect(short.text == "Span 1 min 30 s · rotation 0.9°" && short.hint == "Too short to matter")
        #expect(derotationSummary(info(span: 90, rotation: 1.0)).hint == nil, "one degree is where it starts to matter")
    }

    @Test func aProblemOfTheCoreIsShownInsteadOfTheNumbers() {
        let line = derotationSummary(info(span: 0, rotation: 0, problem: "De-rotation needs the start time of the recording"))
        #expect(line.text == "De-rotation needs the start time of the recording" && line.isProblem && line.hint == nil)
    }

    @Test func theStartTimeIsAlsoShownInUTC() {
        #expect(utcText(Date(timeIntervalSince1970: 1_678_482_000)) == "2023-03-10 21:00 UTC")
    }

    @Test func theDerotationSectionRoundTripsAndKeepsOutOfTheStackSection() throws {
        var stack = StackOptions()
        stack.derotate = true
        stack.derotatePlanet = .mars
        stack.cameraAngle = 33.5
        stack.mirrored = true
        stack.startTime = Date(timeIntervalSince1970: 1_700_000_000.25)
        var back = StackOptions()
        applyDerotationSection(derotationSection(stack), to: &back)
        #expect(back.derotate && back.derotatePlanet == .mars && back.cameraAngle == 33.5 && back.mirrored)
        #expect(back.startTime == stack.startTime)
        // The `stack` section is the one of the Windows app: it does not carry the Mac-only settings.
        guard case .object(let keys) = stackSection(stack) else { throw Missing() }
        #expect(!keys.keys.contains { $0.lowercased().contains("derot") || $0 == "planet" })
    }

    @Test func aProjectWithoutTheSectionLeavesDerotationAlone() {
        var stack = StackOptions()
        stack.derotate = true
        stack.startTime = Date(timeIntervalSince1970: 1_700_000_000)
        applyDerotationSection(nil, to: &stack)
        applyDerotationSection(.string("nope"), to: &stack)
        #expect(stack.derotate && stack.startTime == Date(timeIntervalSince1970: 1_700_000_000))
    }

    @Test func onlyKeysOfTheRightTypeAreTakenFromTheDerotationSection() throws {
        let section = try #require(JSON(text: """
            {"enabled": "yes", "planet": "venus", "cameraAngle": 900, "mirrored": 1, "startTimeUtc": "tomorrow"}
            """))
        var stack = StackOptions()
        stack.startTime = Date(timeIntervalSince1970: 5)
        applyDerotationSection(section, to: &stack)
        #expect(!stack.derotate && stack.derotatePlanet == .jupiter && !stack.mirrored)
        #expect(stack.cameraAngle == 360, "an angle outside the limits is cut to them")
        #expect(stack.startTime == Date(timeIntervalSince1970: 5))
    }
}
