import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// What Stack sends to the core and the texts it shows (same rules as app/src/logic/stack.ts).
@Suite struct StackLogicTests {
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
        #expect(messages.action == "Stack size (n=10) - (1024x680)")
        #expect(messages.result == "Stacking Range : 0-255")
        #expect(stackResultText(frames: 8, width: 1024, height: 680) == "8 frames · 1024×680")
    }

    @Test func aNewSetOfPointsIsCheckedAndHasNoShifts() {
        var data = data()
        #expect(data.pointsChecked == [true, true, true] && data.pointShifts.isEmpty)
        data.pointShifts = [PointShift(dx: 1, dy: 2)]
        data.discardRating()
        #expect(data.pointShifts.isEmpty)
    }
}
